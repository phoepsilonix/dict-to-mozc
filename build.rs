fn main() {
    // for cargo-xwin
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    // "use-snmalloc" feature
    let use_snmalloc = std::env::var("CARGO_FEATURE_USE_SNMALLOC").is_ok();
    let use_tcmalloc_static = std::env::var("CARGO_FEATURE_USE_TCMALLOC_STATIC").is_ok();

    if target_os == "windows" && use_snmalloc {
        // snmalloc
        println!("cargo:rustc-link-arg=/nodefaultlib:libucrt.lib");

        let libs = ["kernel32", "ntdll", "shell32", "user32", "advapi32", "ucrt"];
        for lib in libs {
            println!("cargo:rustc-link-lib={}", lib);
        }
    } else if target_os == "windows" {
        // https://github.com/microsoft/mimalloc/blob/af21001f7a65eafb8fb16460b018ebf9d75e2ad8/CMakeLists.txt#L487
        let libs = ["psapi", "shell32", "user32", "advapi32", "bcrypt"];
        //let libs = ["advapi32"];
        for lib in libs {
            println!("cargo:rustc-link-lib={}", lib);
        }
    }
    if use_tcmalloc_static {
        let out_dir =
            std::path::PathBuf::from(std::env::var_os("OUT_DIR").expect("OUT_DIR was not set"));
        let build_dir = out_dir.join("build");
        println!("cargo:rustc-link-lib=static=tcmalloc");
        println!(
            "cargo:rustc-link-search=native={}/.libs",
            build_dir.display()
        );

        let libs = ["stdc++", "unwind"];
        for lib in libs {
            println!("cargo:rustc-link-lib={}", lib);
        }
    }
}
