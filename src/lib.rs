use std::path::{Path, PathBuf};
pub use projfs_sys as sys;

pub type VersionInfo = *const sys::PRJ_PLACEHOLDER_VERSION_INFO;
pub type Flags = sys::PRJ_CALLBACK_DATA_FLAGS;
pub type Guid = sys::GUID;

extern "C" {
  fn wcslen(ptr: *const std::os::raw::c_ushort) -> usize;
}

pub struct RawPath(sys::PCWSTR);
impl Into<RawPath> for sys::PCWSTR {
  fn into(self) -> RawPath {
    RawPath(self)
  }
}
impl Into<PathBuf> for RawPath {
  fn into(self) -> PathBuf {
    use std::os::windows::prelude::*;
    let ptr = unsafe { std::slice::from_raw_parts(self.0, wcslen(self.0)) };
    std::ffi::OsString::from_wide(ptr).into()
  }
}

pub trait ProjFS {
  fn start_dir_enum(&self, id: Guid, path: RawPath, version: VersionInfo) -> Result<(), sys::HRESULT>;
  fn end_dir_enum(&self, id: Guid, version: VersionInfo) -> Result<(), sys::HRESULT>;
  fn get_dir_enum(&self, id: Guid, path: RawPath, flags: Flags, version: VersionInfo, pattern: RawPath, result_handle: sys::PRJ_DIR_ENTRY_BUFFER_HANDLE) -> Result<(), sys::HRESULT>;

  fn get_metadata(&self, path: RawPath, version: VersionInfo) -> Result<sys::PRJ_PLACEHOLDER_INFO, sys::HRESULT>;

  fn read(&self, path: RawPath, version: VersionInfo, stream: Guid, offset: u64, len: usize) -> Result<(), sys::HRESULT>;
}

mod helper {
  #![allow(non_snake_case)]
  use super::sys::*;
  use super::ProjFS;
  pub trait RawProjFS: ProjFS + Sized {
    unsafe extern "C" fn StartDirectoryEnumerationCallback(arg1: *const PRJ_CALLBACK_DATA, arg2: *const GUID) -> HRESULT {
      let data = arg1.as_ref().unwrap();
      let this = (data.InstanceContext as *mut Self).as_ref().unwrap();
      this.start_dir_enum(*arg2, data.FilePathName.into(), data.VersionInfo).err().unwrap_or_default()
    }
    unsafe extern "C" fn EndDirectoryEnumerationCallback(arg1: *const PRJ_CALLBACK_DATA, arg2: *const GUID) -> HRESULT {
      let data = arg1.as_ref().unwrap();
      let this = (data.InstanceContext as *mut Self).as_ref().unwrap();
      this.end_dir_enum(*arg2, data.VersionInfo).err().unwrap_or_default()
    }
    unsafe extern "C" fn GetDirectoryEnumerationCallback(
      arg1: *const PRJ_CALLBACK_DATA,
      arg2: *const GUID,
      arg3: PCWSTR,
      arg4: PRJ_DIR_ENTRY_BUFFER_HANDLE,
    ) -> HRESULT {
      let data = arg1.as_ref().unwrap();
      let this = (data.InstanceContext as *mut Self).as_ref().unwrap();
      this.get_dir_enum(*arg2, data.FilePathName.into(), data.Flags, data.VersionInfo, arg3.into(), arg4).err().unwrap_or_default()
    }
    unsafe extern "C" fn GetPlaceholderInfoCallback(arg1: *const PRJ_CALLBACK_DATA) -> HRESULT {
      let data = arg1.as_ref().unwrap();
      let this = (data.InstanceContext as *mut Self).as_ref().unwrap();
      match this.get_metadata(data.FilePathName.into(), data.VersionInfo) {
        Ok(result) => {
          PrjWritePlaceholderInfo(data.NamespaceVirtualizationContext, data.FilePathName, &result, std::mem::size_of_val(&result) as u32)
        },
        Err(e) => e,
      }
    }
    unsafe extern "C" fn GetFileDataCallback(arg1: *const PRJ_CALLBACK_DATA, arg2: UINT64, arg3: UINT32) -> HRESULT {
      let data = arg1.as_ref().unwrap();
      let this = (data.InstanceContext as *mut Self).as_ref().unwrap();
      this.read(data.FilePathName.into(), data.VersionInfo, data.DataStreamId, arg2 as u64, arg3 as usize).err().unwrap_or_default()
    }
    // unsafe extern "C" fn QueryFileNameCallback(arg1: *const PRJ_CALLBACK_DATA) -> HRESULT;
    // unsafe extern "C" fn NotificationCallback(
    //   arg1: *const PRJ_CALLBACK_DATA,
    //   arg2: BOOLEAN,
    //   arg3: PRJ_NOTIFICATION,
    //   arg4: PCWSTR,
    //   arg5: *mut PRJ_NOTIFICATION_PARAMETERS,
    // ) -> HRESULT;
    // unsafe extern "C" fn CancelCommandCallback(arg1: *const PRJ_CALLBACK_DATA);
  }
  impl<T: ProjFS> RawProjFS for T { }
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

pub struct Instance {
  raw: sys::PRJ_NAMESPACE_VIRTUALIZATION_CONTEXT,
  cb: sys::PRJ_CALLBACKS,
}

pub fn start_proj_virtualization<P: AsRef<Path>, T: ProjFS + helper::RawProjFS>(path: P, this: Box<T>) -> Result<Instance, sys::HRESULT> {
  use std::os::windows::prelude::*;
  let mut instance = Instance {
    raw: std::ptr::null_mut(),
    cb: trait_to_table::<T>()
  };
  let path = path.as_ref().as_os_str();
  let path_str: Vec<u16> = path.encode_wide().chain(std::iter::once(0)).collect();
  let result = unsafe {
    sys::PrjStartVirtualizing(
      path_str.as_ptr(),
      &instance.cb,
      Box::leak(this) as *const T as *const std::ffi::c_void,
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
