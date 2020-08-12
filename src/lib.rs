use std::path::{Path, PathBuf};
pub use projfs_sys as sys;

pub type CacheMap<T> = chashmap::CHashMap<Guid, Option<std::iter::Peekable<T>>>;

pub type VersionInfo = *const sys::PRJ_PLACEHOLDER_VERSION_INFO;
pub type DirHandle = sys::PRJ_DIR_ENTRY_BUFFER_HANDLE;
pub type Guid = uuid::Uuid;

extern "C" {
  fn wcslen(ptr: *const std::os::raw::c_ushort) -> usize;
}

pub struct RawPath<'a>(sys::PCWSTR, std::marker::PhantomData<&'a Path>);
impl From<sys::PCWSTR> for RawPath<'_> {
  fn from(raw: sys::PCWSTR) -> Self {
    Self(raw, Default::default())
  }
}
impl Into<PathBuf> for RawPath<'_> {
  fn into(self) -> PathBuf {
    use std::os::windows::prelude::*;
    let ptr = unsafe { std::slice::from_raw_parts(self.0, wcslen(self.0)) };
    std::ffi::OsString::from_wide(ptr).into()
  }
}
impl<'a> RawPath<'a> {
  pub fn to_path_buf(self) -> PathBuf {
    self.into()
  }
}

bitflags::bitflags! {
pub struct CallbackDataFlags: sys::PRJ_CALLBACK_DATA_FLAGS {
  const RESTART_SCAN = sys::PRJ_CALLBACK_DATA_FLAGS_PRJ_CB_DATA_FLAG_ENUM_RESTART_SCAN;
  const RETURN_SINGLE_ENTRY = sys::PRJ_CALLBACK_DATA_FLAGS_PRJ_CB_DATA_FLAG_ENUM_RETURN_SINGLE_ENTRY;
}
}

pub fn guid_from_raw(guid: sys::GUID) -> Guid {
  Guid::from_fields(guid.Data1, guid.Data2, guid.Data3, &guid.Data4).expect("guid data4 len")
}

pub fn guid_to_raw(guid: Guid) -> sys::GUID {
  let fields = guid.as_fields();
  sys::GUID {
    Data1: fields.0,
    Data2: fields.1,
    Data3: fields.2,
    Data4: fields.3.clone(),
  }
}

pub fn io_error_to_raw(e: std::io::Error) -> sys::HRESULT {
  use std::io::ErrorKind::*;
  if let Some(i) = e.raw_os_error() {
    return i
  }
  match e.kind() {
    WouldBlock => sys::IO_ERROR_IO_PENDING as sys::HRESULT,
    NotFound => sys::IO_ERROR_FILE_NOT_FOUND as sys::HRESULT,
    InvalidData => sys::IO_ERROR_INSUFFICIENT_BUFFER as sys::HRESULT,
    _ => -1,
  }
}

pub struct FileBasicInfo {
  pub file_name: PathBuf,
  pub is_dir: bool,
  pub file_size: u64,
  pub created: i64,
  pub accessed: i64,
  pub writed: i64,
  pub changed: i64,
  pub attrs: u32,
}

impl AsRef<FileBasicInfo> for FileBasicInfo {
  fn as_ref(&self) -> &Self {
    self
  }
}

impl Into<sys::PRJ_FILE_BASIC_INFO> for &FileBasicInfo {
  fn into(self) -> sys::PRJ_FILE_BASIC_INFO {
    sys::PRJ_FILE_BASIC_INFO {
      IsDirectory: if self.is_dir { 1 } else { 0 },
      ChangeTime: self.changed.into(),
      CreationTime: self.created.into(),
      LastAccessTime: self.accessed.into(),
      LastWriteTime: self.writed.into(),
      FileSize: self.file_size as i64,
      FileAttributes: self.attrs,
    }
  }
}

struct AlignedBuffer(*mut std::ffi::c_void, usize);
impl AlignedBuffer {
  pub fn new(context: sys::PRJ_NAMESPACE_VIRTUALIZATION_CONTEXT, len: usize) -> Self {
    let raw = unsafe { sys::PrjAllocateAlignedBuffer(context, len as u64) };
    Self(raw, len)
  }
  pub fn as_slice_mut(&mut self) -> &mut [u8] {
    unsafe { std::slice::from_raw_parts_mut(self.0 as *mut _, self.1) }
  }
}
impl Drop for AlignedBuffer {
  fn drop(&mut self) {
    unsafe { sys::PrjFreeAlignedBuffer(self.0) }
  }
}

pub trait ProjFSDirEnum {
  type DirIter: Iterator<Item=FileBasicInfo>;
  fn dir_iter(&self, id: Guid, path: RawPath, pattern: Option<RawPath>, version: VersionInfo) -> std::io::Result<Self::DirIter>;
  fn dir_iter_cache(&self, version: VersionInfo) -> &CacheMap<Self::DirIter>;
}

pub trait ProjFSRead {
  fn get_metadata(&self, path: RawPath, version: VersionInfo) -> std::io::Result<FileBasicInfo>;
  fn read(&self, path: RawPath, version: VersionInfo, offset: u64, buf: &mut [u8]) -> std::io::Result<()>;
}

impl<T: ProjFSDirEnum + ProjFSRead> ProjFS for T {
  fn start_dir_enum(&self, id: Guid, _path: RawPath, version: VersionInfo) -> std::io::Result<()> {
    self.dir_iter_cache(version).insert_new(id, None); Ok(())
  }
  fn end_dir_enum(&self, id: Guid, version: VersionInfo) -> std::io::Result<()> {
    self.dir_iter_cache(version).remove(&id); Ok(())
  }
  fn get_dir_enum(&self, id: Guid, path: RawPath, flags: CallbackDataFlags, version: VersionInfo, pattern: Option<RawPath>, handle: DirHandle) -> std::io::Result<()> {
    let cache = self.dir_iter_cache(version);
    let mut dir_iter = &mut cache.get_mut(&id).ok_or(std::io::ErrorKind::InvalidData)?;
    let dir_iter: &mut Option<_> = &mut dir_iter;
    if dir_iter.is_none() || flags.contains(CallbackDataFlags::RESTART_SCAN) {
      dir_iter.replace(self.dir_iter(id, path, pattern, version)?.peekable());
    }
    if let Some(ref mut dir_iter) = dir_iter {
      Self::fill_entries(dir_iter, handle);
    }
    Ok(())
  }

  fn get_metadata(&self, path: RawPath, version: VersionInfo) -> std::io::Result<FileBasicInfo> {
    ProjFSRead::get_metadata(self, path, version)
  }

  fn read(&self, path: RawPath, version: VersionInfo, offset: u64, buf: &mut [u8]) -> std::io::Result<()> {
    ProjFSRead::read(self, path, version, offset, buf)
  }
}

pub trait ProjFS {
  fn start_dir_enum(&self, id: Guid, path: RawPath, version: VersionInfo) -> std::io::Result<()>;
  fn end_dir_enum(&self, id: Guid, version: VersionInfo) -> std::io::Result<()>;
  fn get_dir_enum(&self, id: Guid, path: RawPath, flags: CallbackDataFlags, version: VersionInfo, pattern: Option<RawPath>, handle: DirHandle) -> std::io::Result<()>;

  fn fill_entries<'a, I: AsRef<FileBasicInfo>, Iter: Iterator<Item=I>>(iter: &mut std::iter::Peekable<Iter>, handle: DirHandle) -> usize {
    let mut k = 0;
    while let Some(i) = iter.peek() {
      use std::os::windows::ffi::OsStrExt;
      let i = i.as_ref();
      let mut basic_info = i.into();
      let file_name: Vec<u16> = i.file_name.as_os_str().encode_wide().chain(std::iter::once(0)).collect();
      let hr = unsafe { sys::PrjFillDirEntryBuffer(file_name.as_ptr(), &mut basic_info, handle) };
      if hr == 0 { k += 1; } else { return k }
      iter.next();
    }
    k
  }

  fn get_metadata(&self, path: RawPath, version: VersionInfo) -> std::io::Result<FileBasicInfo>;

  fn read(&self, path: RawPath, version: VersionInfo, offset: u64, buf: &mut [u8]) -> std::io::Result<()>;
}

mod helper {
  #![allow(non_snake_case)]
  use super::sys::*;
  use super::*;
  pub trait RawProjFS: ProjFS + Sized {
    unsafe extern "C" fn StartDirectoryEnumerationCallback(arg1: *const PRJ_CALLBACK_DATA, arg2: *const GUID) -> HRESULT {
      let data = arg1.as_ref().unwrap();
      let this = (data.InstanceContext as *mut Self).as_ref().unwrap();
      let result = this.start_dir_enum(guid_from_raw(*arg2), data.FilePathName.into(), data.VersionInfo);
      match result {
        Ok(()) => 0,
        Err(e) => io_error_to_raw(e)
      }
      // ERROR_FILE_NOT_FOUND
    }
    unsafe extern "C" fn EndDirectoryEnumerationCallback(arg1: *const PRJ_CALLBACK_DATA, arg2: *const GUID) -> HRESULT {
      let data = arg1.as_ref().unwrap();
      let this = (data.InstanceContext as *mut Self).as_ref().unwrap();
      let result = this.end_dir_enum(guid_from_raw(*arg2), data.VersionInfo);
      match result {
        Ok(()) => 0,
        Err(e) => io_error_to_raw(e)
      }
    }
    unsafe extern "C" fn GetDirectoryEnumerationCallback(
      arg1: *const PRJ_CALLBACK_DATA,
      arg2: *const GUID,
      arg3: PCWSTR,
      arg4: PRJ_DIR_ENTRY_BUFFER_HANDLE,
    ) -> HRESULT {
      let data = arg1.as_ref().unwrap();
      let this = (data.InstanceContext as *mut Self).as_ref().unwrap();
      let result = this.get_dir_enum(
        guid_from_raw(*arg2),
        data.FilePathName.into(),
        CallbackDataFlags::from_bits(data.Flags).unwrap(),
        data.VersionInfo,
        if arg3 == std::ptr::null() { None } else { Some(arg3.into()) },
        arg4
      );
      match result {
        Ok(()) => 0,
        Err(e) => io_error_to_raw(e)
      }
      // ERROR_INSUFFICIENT_BUFFER
    }
    unsafe extern "C" fn GetPlaceholderInfoCallback(arg1: *const PRJ_CALLBACK_DATA) -> HRESULT {
      let data = arg1.as_ref().unwrap();
      let this = (data.InstanceContext as *mut Self).as_ref().unwrap();
      match this.get_metadata(data.FilePathName.into(), data.VersionInfo) {
        Ok(result) => {
          let mut placeholder_info: sys::PRJ_PLACEHOLDER_INFO = std::mem::zeroed();
          placeholder_info.FileBasicInfo = (&result).into();
          PrjWritePlaceholderInfo(data.NamespaceVirtualizationContext, data.FilePathName, &placeholder_info, std::mem::size_of_val(&placeholder_info) as u32)
        },
        Err(e) => io_error_to_raw(e),
      }
      // ERROR_FILE_NOT_FOUND
    }
    unsafe extern "C" fn GetFileDataCallback(arg1: *const PRJ_CALLBACK_DATA, arg2: UINT64, arg3: UINT32) -> HRESULT {
      let data = arg1.as_ref().unwrap();
      let this = (data.InstanceContext as *mut Self).as_ref().unwrap();
      let mut buf = AlignedBuffer::new(data.NamespaceVirtualizationContext, arg3 as usize);
      let result = this.read(data.FilePathName.into(), data.VersionInfo, arg2, buf.as_slice_mut());
      match result {
        Ok(()) => {
          sys::PrjWriteFileData(data.NamespaceVirtualizationContext, &data.DataStreamId, buf.0, arg2, arg3)
        },
        Err(e) => io_error_to_raw(e)
      }
      // S_OK, ERROR_IO_PENDING
    }
    // unsafe extern "C" fn QueryFileNameCallback(arg1: *const PRJ_CALLBACK_DATA) -> HRESULT; // ERROR_FILE_NOT_FOUND
    // unsafe extern "C" fn NotificationCallback(
    //   arg1: *const PRJ_CALLBACK_DATA,
    //   arg2: BOOLEAN,
    //   arg3: PRJ_NOTIFICATION,
    //   arg4: PCWSTR,
    //   arg5: *mut PRJ_NOTIFICATION_PARAMETERS,
    // ) -> HRESULT;
    // unsafe extern "C" fn CancelCommandCallback(arg1: *const PRJ_CALLBACK_DATA);
  }
  impl<T: ProjFS + Sync> RawProjFS for T { }
}

fn trait_to_table<T: helper::RawProjFS>() -> sys::PRJ_CALLBACKS {
  let cb = sys::PRJ_CALLBACKS {
    StartDirectoryEnumerationCallback: Some(T::StartDirectoryEnumerationCallback),
    EndDirectoryEnumerationCallback: Some(T::EndDirectoryEnumerationCallback),
    GetDirectoryEnumerationCallback: Some(T::GetDirectoryEnumerationCallback),
    GetPlaceholderInfoCallback: Some(T::GetPlaceholderInfoCallback),
    GetFileDataCallback: Some(T::GetFileDataCallback),
    QueryFileNameCallback: None, //Some(T::QueryFileNameCallback),
    NotificationCallback: None, //Some(T::NotificationCallback),
    CancelCommandCallback: None, //Some(T::CancelCommandCallback),
  };
  cb
}

pub struct Instance<T> {
  raw: sys::PRJ_NAMESPACE_VIRTUALIZATION_CONTEXT,
  this: *mut T,
  cb: sys::PRJ_CALLBACKS,
}

pub fn start_proj_virtualization<P: AsRef<Path>, T: ProjFS + Sync>(path: P, this: Box<T>) -> Result<Instance<T>, sys::HRESULT> {
  use std::os::windows::prelude::*;
  let mut instance = Instance {
    raw: std::ptr::null_mut(),
    this: Box::leak(this),
    cb: trait_to_table::<T>()
  };
  let path = path.as_ref().canonicalize().unwrap();
  let path_str: Vec<u16> = path.as_os_str().encode_wide().chain(std::iter::once(0)).collect();
  let result = unsafe {
    // let id = uuid::Uuid::new_v5(uuid::Uuid::NAMESPACE_URL, std::slice::from_raw_parts(path_str.as_ptr(), path_str.len()*2));
    let id = uuid::Uuid::new_v4();
    sys::PrjMarkDirectoryAsPlaceholder(
      path_str.as_ptr(),
      std::ptr::null(),
      std::ptr::null(),
      &guid_to_raw(id),
    );
    sys::PrjStartVirtualizing(
      path_str.as_ptr(),
      &instance.cb,
      instance.this as *const std::ffi::c_void,
      std::ptr::null(),
      &mut instance.raw
    )
  };
  if result == 0 {
    Ok(instance)
  } else {
    Err(result)
  }
}

impl<T> Drop for Instance<T> {
  fn drop(&mut self) {
    unsafe { Box::from_raw(self.this) };
    if self.raw != std::ptr::null_mut() {
      unsafe { sys::PrjStopVirtualizing(self.raw) }
    }
  }
}
