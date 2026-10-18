fn main() {
    #[cfg(target_os = "linux")]
    {
        println!("cargo:rustc-cdylib-link-arg=-Wl,-soname,libfreetype.so.6");
        println!("cargo:rustc-cdylib-link-arg=-Wl,--version-script=crates/honey-shim/freetype.version");
        println!("cargo:rustc-cdylib-link-arg=-Wl,--undefined-version");
    }
}
