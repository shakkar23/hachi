fn main() {
    println!("cargo:rustc-env=DUCKDB_INCLUDE_DIR=C:/lib/duckdb");
    
    println!("cargo:rustc-link-search=native=C:/lib/duckdb");
    println!("cargo:rustc-link-lib=static=duckdb");

    println!("cargo:rustc-link-lib=rstrtmgr");
}