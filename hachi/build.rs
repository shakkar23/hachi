fn main() {
    println!("cargo:rustc-link-search=native=/root/hachi");
    println!("cargo:rustc-link-lib=dylib=lib_lightgbm");
}