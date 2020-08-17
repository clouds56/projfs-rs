#[cfg(feature = "bindgen")]
const fn target_arch() -> Option<&'static str> {
  #[cfg(not(target_os="windows"))]
  { None }
  #[cfg(target_os="windows")] {
    #[cfg(target_arch="x86")] {"x86"}
    #[cfg(target_arch="x86_64")] {"x64"}
    #[cfg(target_arch="aarch64")] {"arm64"}
    #[cfg(target_arch="arm")] {"arm"}
  }.into()
}

#[cfg(feature = "bindgen")]
fn gen_bindings() {
  // The bindgen::Builder is the main entry point
  // to bindgen, and lets you build up options for
  // the resulting bindings.
  let bindings = bindgen::Builder::default()
    // The input header we would like to generate
    // bindings for.
    .header("wrapper.h")
    // Tell cargo to invalidate the built crate whenever any of the
    // included header files changed.
    .parse_callbacks(Box::new(bindgen::CargoCallbacks));
  let bindings = [
    "PrjMarkDirectoryAsPlaceholder",
    "PrjStartVirtualizing",
    "PrjStopVirtualizing",
    "PrjFillDirEntryBuffer",
    "PrjFileNameMatch",
    "PrjWritePlaceholderInfo",
    "PrjWriteFileData",
    "PrjAllocateAlignedBuffer",
    "PrjFreeAlignedBuffer",
  ].iter().fold(bindings, |b, s| b.whitelist_function(s));
  let bindings = bindings
    .whitelist_type("IO_ERROR")
    // UB: https://github.com/rust-lang/rust/issues/36927
    // .rustified_non_exhaustive_enum("IO_ERROR")
    // Finish the builder and generate the bindings.
    .generate()
    // Unwrap the Result and panic on failure.
    .expect("Unable to generate bindings");

  // Write the bindings to the $OUT_DIR/bindings.rs file.
  // let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
  let out_path = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
  bindings
    .write_to_file(out_path.join("bindings.rs"))
    .expect("Couldn't write bindings!");
}

#[cfg(not(feature = "bindgen"))]
fn gen_bindings() { }

fn main() {
  // Tell cargo to tell rustc to link the system bzip2
  // shared library.
  println!("cargo:rustc-link-lib=ProjectedFSLib");
  #[cfg(feature = "bindgen")]
  if let Some(arch) = target_arch() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    println!("cargo:rustc-link-search={}/lib/{}", manifest_dir, arch);
  }

  // Tell cargo to invalidate the built crate whenever the wrapper changes
  println!("cargo:rerun-if-changed=wrapper.h");
  println!("cargo:rerun-if-changed=projectedfslib.h");

  gen_bindings()
}
