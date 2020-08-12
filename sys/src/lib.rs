#[allow(non_camel_case_types, non_snake_case)]
mod bindings {
  #[cfg(not(feature = "bindgen"))]
  include!("bindings.rs");

  #[cfg(feature = "bindgen")]
  include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}
pub use bindings::*;

impl From<i64> for LARGE_INTEGER {
  fn from(i: i64) -> Self {
    Self { QuadPart: i }
  }
}

impl Into<i64> for LARGE_INTEGER {
  fn into(self) -> i64 {
    unsafe { self.QuadPart }
  }
}
