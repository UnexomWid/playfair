use sha2::{Digest, Sha256};
use std::fs::read;

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
