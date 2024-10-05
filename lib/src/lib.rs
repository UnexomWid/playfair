#![deny(clippy::all)]

use base64::prelude::*;
use memmap2::Mmap;
use obfstr::obfstr;
use sha2::{Digest, Sha256};
use std::error::Error;
use std::fs::{read, File};
use std::path::Path;
use std::result::Result;
use std::str::from_utf8;
use url::Url;

use napi::{Env, JsObject};

#[macro_use]
extern crate napi_derive;

unsafe fn msg(what: &str) {
  println!("{}", what);
}

// Node.js expects absolute and URL-encoded paths for import references from code imported in memory
// ./file.js ---> file:///.../file.js
fn fix_imports(src: &str) -> String {
  let mut out = String::with_capacity(src.len());

  for line in src.lines() {
    let line = line.trim();

    if line.len() > "import ".len() {
      // This handles all cases:
      // import stuff
      // import{stuff}
      // import { stuff }
      let separator = line.chars().nth("import".len()).unwrap();

      if line.starts_with("import") && (separator == ' ' || separator == '{') {
        if let Some(quote_start) = line.find('\'').or_else(|| line.find('"')) {
          let quote = line.chars().nth(quote_start).unwrap();

          if let Some(quote_end) = line.rfind(quote) {
            let path = &line[quote_start + 1..quote_end];

            if path.starts_with(".") {
              // The working directory MUST be set to the dir where the original script was located.
              // This is done automatically in the JS wrapper via process.chdir()
              let path = std::env::current_dir().unwrap().join(&path);

              out.push_str(&line[..quote_start + 1]);
              out.push_str(Url::from_file_path(path).unwrap().as_str());
              out.push_str(&line[quote_end..]);
              out.push('\n');

              continue;
            }
          }
        }
      }
    }

    out.push_str(&line);
    out.push('\n');
  }

  out
}

fn load() -> Result<String, Box<dyn Error>> {
  // Check wrapper file hash
  let path = "./.package/package.mjs";

  if !Path::new(path).exists() {
    return Err("Wrapper file does not exist".into());
  }

  let mut hash = Sha256::new();

  hash.update(read(path)?);

  let hash = hex::encode_upper(&hash.finalize()[..]);

  // TODO: Timing-safe
  if hash != obfstr!(env!("PLAYFAIR_JS_WRAPPER_HASH")) {
    return Err("Wrapper file integrity check failed".into());
  }

  // Unpack
  let path = "./.package/package.dat";

  if !Path::new(path).exists() {
    return Err("File does not exist".into());
  }

  let file = File::open(path)?;

  let map = unsafe { Mmap::map(&file)? };

  let out = playfair_core::unpack(&map, None)?;
  let out = fix_imports(from_utf8(&out)?);

  Ok(format!(
    "data:text/javascript;base64,{}",
    String::from(BASE64_STANDARD.encode(out.as_bytes()))
  ))
}

#[module_exports]
unsafe fn module_exports(mut exports: JsObject, env: Env) -> napi::Result<()> {
  let binding = load();

  if binding.is_err() {
    msg("Tampering detected! You are not playing fair, so neither am I.");

    // let mut err_file = File::options()
    //         .read(true)
    //         .write(true)
    //         .create(true)
    //         .truncate(true)
    //         .open("./_err.log")?;

    // err_file.write_all(binding.unwrap_err().to_string().as_bytes())?;

    std::process::exit(1);
  }

  exports.set_named_property("binding", env.create_string(binding.unwrap().as_str()))?;

  Ok(())
}
