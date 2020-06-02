use std::path::PathBuf;
use projfs::*;
use chashmap::CHashMap;

pub struct DirInfo {
  path: PathBuf,
  cache: Option<Vec<FileBasicInfo>>,
  idx: usize,
}
impl DirInfo {
  fn new(path: PathBuf) -> Self {
    Self {
      path, idx: 0, cache: None
    }
  }
}

#[derive(Default)]
pub struct MyProjFS {
  dir_enums: CHashMap<Guid, DirInfo>,
}
impl ProjFS for MyProjFS {
  fn start_dir_enum(&self, id: Guid, path: RawPath, _: VersionInfo) -> Result<(), i32> {
    let path: PathBuf = path.into();
    println!("start_dir_enum: {:?} {}", path.display(), id);
    let _ = self.dir_enums.insert(id, DirInfo::new(path));
    Ok(())
  }
  fn end_dir_enum(&self, id: Guid, _: VersionInfo) -> Result<(), i32> {
    self.dir_enums.remove(&id);
    Ok(())
  }
  fn get_dir_enum(&self, id: Guid, path: RawPath, flags: CallbackDataFlags, _: VersionInfo, pattern: RawPath, handle: DirHandle) -> Result<(), i32> {
    println!("get_dir_enum: {:?} {} {:?} {:?}", path.to_path_buf().display(), id, flags, pattern.to_path_buf().display());
    let mut dir_info = self.dir_enums.get_mut(&id).unwrap();
    if dir_info.cache.is_none() || flags.contains(CallbackDataFlags::RESTART_SCAN) {
      dir_info.cache = Some(vec![]);
      dir_info.idx = 0
    }
    if let Some(cache) = &dir_info.cache {
      let k = Self::fill_entries(cache[dir_info.idx..].iter(), handle);
      dir_info.idx += k;
      println!("fill {} entries", k);
      Ok(())
    } else {
      Err(0)
    }
  }
  fn get_metadata(&self, path: RawPath, _: VersionInfo) -> std::result::Result<sys::PRJ_PLACEHOLDER_INFO, i32> {
    println!("read metadata {:?}", path.to_path_buf().display());
    Err(0)
  }
  fn read(&self, _: RawPath, _: VersionInfo, _: Guid, _: u64, _: usize) -> std::result::Result<(), i32> { unimplemented!() }
}

fn main() {
  std::fs::create_dir("test_dir").ok();
  let instance = start_proj_virtualization("test_dir", Box::new(MyProjFS::default())).unwrap();
  std::thread::sleep(std::time::Duration::from_secs(std::u64::MAX));
  drop(instance)
}
