fn main() {
    println!("cargo:warning=Linking rstrtmgr for DuckDB lock info");
    println!("cargo:rustc-link-lib=rstrtmgr");
}