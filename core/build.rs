fn main() {
  println!("cargo:rustc-env=PLAYFAIR_MAGIC=FAIR");
  println!("cargo:rustc-env=PLAYFAIR_HMAC_KEY=FAIR");
  println!("cargo:rustc-env=PLAYFAIR_USER_AGENT=Mozilla/5.0 (Windows; Windows NT 6.2;) AppleWebKit/601.34 (KHTML, like Gecko) Chrome/52.0.2897.196 Safari/535.0 Edge/13.33431");
}
