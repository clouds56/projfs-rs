use projfs::*;

pub struct MyProjFS();
impl ProjFS for MyProjFS {
  fn start_dir_enum(&self, _: Guid, _: RawPath, _: VersionInfo) -> Result<(), i32> { unimplemented!() }
  fn end_dir_enum(&self, _: Guid, _: VersionInfo) -> Result<(), i32> { unimplemented!() }
  fn get_dir_enum(&self, _: Guid, _: RawPath, _: i32, _: VersionInfo, _: RawPath, _: sys::PRJ_DIR_ENTRY_BUFFER_HANDLE) -> Result<(), i32> { unimplemented!() }
  fn get_metadata(&self, _: RawPath, _: VersionInfo) -> std::result::Result<sys::PRJ_PLACEHOLDER_INFO, i32> { unimplemented!() }
  fn read(&self, _: RawPath, _: VersionInfo, _: Guid, _: u64, _: usize) -> std::result::Result<(), i32> { unimplemented!() }
}

fn main() {
  let _ = start_proj_virtualization("test_dir", Box::new(MyProjFS()));
}
