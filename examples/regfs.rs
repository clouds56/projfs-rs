use std::path::{Path, PathBuf};
use projfs::*;
use chashmap::CHashMap;
use std::sync::Mutex;
use winreg::enums::*;
use winreg::{RegKey, RegValue};

pub struct DirInfo {
  key: Mutex<RegKey>,
  cache: Option<Vec<FileBasicInfo>>,
  idx: usize,
}
impl DirInfo {
  fn new(root: &RegKey, path: PathBuf) -> std::io::Result<Self> {
    Ok(Self {
      key: Mutex::new(root.open_subkey(path)?), idx: 0, cache: None
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
  dir_enums: CHashMap<Guid, DirInfo>,
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
}
impl ProjFS for MyProjFS {
  fn start_dir_enum(&self, id: Guid, path: RawPath, _: VersionInfo) -> std::io::Result<()> {
    let path: PathBuf = path.into();
    println!("start_dir_enum: {:?} {}", path.display(), id);
    let _ = self.dir_enums.insert(id, DirInfo::new(&self.reg_root.lock().unwrap(), path)?);
    Ok(())
  }
  fn end_dir_enum(&self, id: Guid, _: VersionInfo) -> std::io::Result<()> {
    self.dir_enums.remove(&id);
    Ok(())
  }
  fn get_dir_enum(&self, id: Guid, path: RawPath, flags: CallbackDataFlags, _: VersionInfo, pattern: Option<RawPath>, handle: DirHandle) -> std::io::Result<()> {
    println!("get_dir_enum: {:?} {} {:?} {:?}", path.to_path_buf().display(), id, flags, pattern.map(|i| i.to_path_buf()));
    let mut dir_info = self.dir_enums.get_mut(&id).unwrap();
    if dir_info.cache.is_none() || flags.contains(CallbackDataFlags::RESTART_SCAN) {
      let keys = dir_info.get_subkeys();
      let values = dir_info.get_subvalues();
      println!("found {} + {} entries", keys.len(), values.len());
      dir_info.cache = Some(keys.into_iter().chain(values).collect());
      dir_info.idx = 0
    }
    if let Some(cache) = &dir_info.cache {
      let k = Self::fill_entries(cache[dir_info.idx..].iter(), handle);
      println!("fill {} entries out of {}..{}", k, dir_info.idx, cache.len());
      dir_info.idx += k;
      Ok(())
    } else {
      Err(std::io::ErrorKind::NotFound.into())
    }
  }
  fn get_metadata(&self, path: RawPath, _: VersionInfo) -> std::io::Result<FileBasicInfo> {
    let path = path.to_path_buf();
    println!("read metadata {:?}", path.display());
    fn open_subvalue(root: &RegKey, path: &Path) -> Option<RegValue> {
      let parent = path.parent()?;
      let file_name = path.file_name()?;
      let key = root.open_subkey(parent).ok()?;
      key.get_raw_value(file_name).ok()
    }
    let root_reg = self.reg_root.lock().unwrap();
    let size = if root_reg.open_subkey(&path).is_ok() {
      None
    } else if let Some(value) = open_subvalue(&root_reg, &path) {
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
  fn read(&self, _: RawPath, _: VersionInfo, _: Guid, _: u64, _: usize) -> std::io::Result<()> { unimplemented!() }
}

fn main() {
  std::fs::create_dir("test_dir").ok();
  let instance = start_proj_virtualization("test_dir", Box::new(MyProjFS::new())).unwrap();
  std::thread::sleep(std::time::Duration::from_secs(std::u64::MAX));
  drop(instance)
}
