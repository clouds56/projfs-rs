extern crate bindgen;

fn main() {
  // Tell cargo to tell rustc to link the system bzip2
  // shared library.
  println!("cargo:rustc-link-lib=ProjectedFSLib");

  // Tell cargo to invalidate the built crate whenever the wrapper changes
  println!("cargo:rerun-if-changed=wrapper.h");

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
    .derive_hash(true)
    .derive_eq(true)
    // Finish the builder and generate the bindings.
    .generate()
    // Unwrap the Result and panic on failure.
    .expect("Unable to generate bindings");

  // Write the bindings to the $OUT_DIR/bindings.rs file.
  // let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
  let out_path = std::path::Path::new("src");
  bindings
    .write_to_file(out_path.join("bindings.rs"))
    .expect("Couldn't write bindings!");
}
