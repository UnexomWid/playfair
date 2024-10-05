use std::fs::read;
use sha2::{Sha256, Digest};

extern crate napi_build;

fn main() {
  let data = read("./index.js").unwrap();

  let mut hash = Sha256::new();

  hash.update(data);

  println!(
      "cargo:rustc-env=PLAYFAIR_JS_WRAPPER_HASH={}",
      hex::encode_upper(&hash.finalize()[..])
  );

  napi_build::setup();
}
