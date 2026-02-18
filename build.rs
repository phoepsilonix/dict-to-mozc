fn main() {
    // for cargo-xwin
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    if target_os == "windows" {
        // https://github.com/microsoft/mimalloc/blob/af21001f7a65eafb8fb16460b018ebf9d75e2ad8/CMakeLists.txt#L487
        let libs = ["psapi", "shell32", "user32", "advapi32", "bcrypt"];
        //let libs = ["advapi32"];
        for lib in libs {
            println!("cargo:rustc-link-lib={}", lib);
        }
    }
}
