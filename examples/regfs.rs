use log::*;
use std::path::PathBuf;
use projfs::*;
use std::sync::RwLock;
use std::collections::HashMap;

pub struct DirInfo {
  path: PathBuf,
  idx: usize,
}
impl DirInfo {
  fn new(path: PathBuf) -> Self {
    Self {
      path, idx: 0
    }
  }
}

#[derive(Default)]
pub struct MyProjFS {
  dir_enums: RwLock<HashMap<Guid, DirInfo>>,
}
impl ProjFS for MyProjFS {
  fn start_dir_enum(&self, id: Guid, path: RawPath, _: VersionInfo) -> Result<(), i32> {
    let path: PathBuf = path.into();
    println!("start_dir_enum: {}", path.display());
    let _ = self.dir_enums.write().unwrap().insert(id, DirInfo::new(path));
    Ok(())
  }
  fn end_dir_enum(&self, id: Guid, _: VersionInfo) -> Result<(), i32> {
    self.dir_enums.write().unwrap().remove(&id);
    Ok(())
  }
  fn get_dir_enum(&self, id: Guid, _: RawPath, _: i32, _: VersionInfo, _: RawPath, _: sys::PRJ_DIR_ENTRY_BUFFER_HANDLE) -> Result<(), i32> {
    let dir_info = self.dir_enums.read().unwrap().get(&id);
    Ok(())
  }
  fn get_metadata(&self, _: RawPath, _: VersionInfo) -> std::result::Result<sys::PRJ_PLACEHOLDER_INFO, i32> { unimplemented!() }
  fn read(&self, _: RawPath, _: VersionInfo, _: Guid, _: u64, _: usize) -> std::result::Result<(), i32> { unimplemented!() }
}

fn main() {
  std::fs::create_dir("test_dir").ok();
  start_proj_virtualization("test_dir", Box::new(MyProjFS::default())).unwrap();
  std::thread::sleep(std::time::Duration::from_secs(std::u64::MAX));
}
