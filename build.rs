fn main() {
    println!("cargo:rustc-link-search=native=/usr/lib");
    println!("cargo:rustc-link-search=native=/usr/local/lib");
    
    println!("cargo:rerun-if-changed=build.rs");
}