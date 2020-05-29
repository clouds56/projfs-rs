use projfs::*;

pub struct MyProjFS();
impl ProjFS for MyProjFS {

}

fn main() {
  let _ = start_proj_virtualization("test_dir", Box::new(MyProjFS()));
}
