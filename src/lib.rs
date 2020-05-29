use std::path::Path;
pub use projfs_sys as sys;

pub trait ProjFS {

}

fn trait_to_table<T: ProjFS>() -> *const sys::PRJ_CALLBACKS {
  unimplemented!()
}

pub struct Instance {
  raw: sys::PRJ_NAMESPACE_VIRTUALIZATION_CONTEXT
}

pub fn start_proj_virtualization<P: AsRef<Path>, T: ProjFS>(path: P, this: Box<T>) -> Result<Instance, sys::HRESULT> {
  use std::os::windows::prelude::*;
  let mut instance = Instance { raw: std::ptr::null_mut() };
  let path = path.as_ref().as_os_str();
  let path_str: Vec<u16> = path.encode_wide().chain(std::iter::once(0)).collect();
  let result = unsafe {
    sys::PrjStartVirtualizing(
      path_str.as_ptr(),
      trait_to_table::<T>(),
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
