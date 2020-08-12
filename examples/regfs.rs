use std::path::{Path, PathBuf};
use projfs::*;
use std::sync::Mutex;
use winreg::enums::*;
use winreg::{RegKey, RegValue};

pub struct DirInfo {
  key: Mutex<RegKey>,
}
impl DirInfo {
  fn new(root: &RegKey, path: PathBuf) -> std::io::Result<Self> {
    Ok(Self {
      key: Mutex::new(root.open_subkey(path)?),
    })
  }
  fn get_subkeys(&self) -> Vec<FileBasicInfo> {
    let key = self.key.lock().unwrap();
    key.enum_keys()
      .filter_map(|n| {
        let n = n.ok()?;
        FileBasicInfo {
          file_name: n.into(),
          file_size: 0,
          is_dir: true,
          created: 0, writed: 0, changed: 0, accessed: 0,
          attrs: 0,
        }.into()
      }).collect()
  }
  fn get_subvalues(&self) -> Vec<FileBasicInfo> {
    let key = self.key.lock().unwrap();
    key.enum_values()
      .filter_map(|n| {
        let (n, v) = n.ok()?;
        FileBasicInfo {
          file_name: n.into(),
          file_size: v.bytes.len() as u64,
          is_dir: false,
          created: 0, writed: 0, changed: 0, accessed: 0,
          attrs: 0,
        }.into()
      }).collect()
  }
}

pub struct MyProjFS {
  dir_enums: CacheMap<Box<dyn Iterator<Item=FileBasicInfo> + Send + Sync>>,
  reg_root: Mutex<RegKey>,
}
impl MyProjFS {
  fn new() -> Self {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    Self {
      dir_enums: Default::default(),
      reg_root: Mutex::new(hklm),
    }
  }
  fn open_subvalue(root: &RegKey, path: &Path) -> Option<RegValue> {
    let parent = path.parent()?;
    let file_name = path.file_name()?;
    let key = root.open_subkey(parent).ok()?;
    key.get_raw_value(file_name).ok()
  }
}

impl ProjFSDirEnum for MyProjFS {
  type DirIter = Box<dyn Iterator<Item=FileBasicInfo> + Send + Sync>;
  fn dir_iter(&self, _id: Guid, path: RawPath, _pattern: Option<RawPath>, _version: VersionInfo) -> std::io::Result<Self::DirIter> {
    let dir_info = DirInfo::new(&self.reg_root.lock().unwrap(), path.into())?;
    let keys = dir_info.get_subkeys();
    let values = dir_info.get_subvalues();
    println!("found {} + {} entries", keys.len(), values.len());
    Ok(Box::new(keys.into_iter().chain(values)))
  }
  fn dir_iter_cache(&self, _version: VersionInfo) -> &CacheMap<Self::DirIter> {
    &self.dir_enums
  }
}
impl ProjFSRead for MyProjFS {
  fn get_metadata(&self, path: RawPath, _: VersionInfo) -> std::io::Result<FileBasicInfo> {
    let path = path.to_path_buf();
    println!("read metadata {:?}", path.display());
    let root_reg = self.reg_root.lock().unwrap();
    let size = if root_reg.open_subkey(&path).is_ok() {
      None
    } else if let Some(value) = Self::open_subvalue(&root_reg, &path) {
      Some(value.bytes.len() as u64)
    } else {
      return Err(std::io::ErrorKind::NotFound.into())
    };
    let result = FileBasicInfo {
      file_name: path,
      file_size: size.unwrap_or_default(),
      is_dir: size.is_none(),
      created: 0, writed: 0, changed: 0, accessed: 0,
      attrs: 0,
    };
    Ok(result)
  }
  fn read(&self, path: RawPath, _: VersionInfo, offset: u64, buf: &mut [u8]) -> std::io::Result<()> {
    let path = path.to_path_buf();
    println!("read content {:?} {}-{}", path.display(), offset, offset + buf.len() as u64);
    if let Some(value) = Self::open_subvalue(&self.reg_root.lock().unwrap(), &path) {
      buf.copy_from_slice(&value.bytes[offset as usize..offset as usize + buf.len()]);
      Ok(())
    } else {
      return Err(std::io::ErrorKind::NotFound.into())
    }
  }
}

fn main() {
  std::fs::create_dir("test_dir").ok();
  let instance = start_proj_virtualization("test_dir", Box::new(MyProjFS::new())).unwrap();
  std::thread::sleep(std::time::Duration::from_secs(std::u64::MAX));
  drop(instance)
}
