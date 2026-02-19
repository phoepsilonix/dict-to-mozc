fn main() {
    // for cargo-xwin
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let target_env = std::env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();
    //let target_arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();

    // "use-snmalloc" feature
    //let use_snmalloc = std::env::var("CARGO_FEATURE_USE_SNMALLOC").is_ok();
    let use_snmalloc = std::env::var("CARGO_FEATURE_USE_SNMALLOC").is_ok() || std::env::var("CARGO_FEATURE_USE_SNMALLOC_CC").is_ok();

    let use_tcmalloc_static = std::env::var("CARGO_FEATURE_USE_TCMALLOC_STATIC").is_ok();
    let use_mimalloc = std::env::var("CARGO_FEATURE_USE_MIMALLOC").is_ok();

    if target_os == "linux" && use_tcmalloc_static {
        // lzma.a
        // pacman-static
        //println!("cargo:rustc-link-search=native={}", "/usr/lib/pacman/lib");
        //println!("cargo:rustc-link-lib=static=lzma");
        
        // libunwind.a
        // rust-musl or rust-aarch64-musl 
        //println!("cargo:rustc-link-search=native={}{}{}", "/usr/lib/rustlib/", target_arch, "-unknown-linux-musl/lib/self-contained/");
        //println!("cargo:rustc-link-search=native={}{}{}", "/usr/lib/rustlib/", target_arch, "-unknown-linux-musl/lib");
        //println!("cargo:rustc-link-lib=static=unwind");

        //println!("cargo:rustc-link-search=native={}", out_dir.display());
        
        // static link
        let libs = ["tcmalloc"];
        for lib in libs {
            println!("cargo:rustc-link-lib=static={}", lib);
        }

        let libs = ["stdc++", "unwind", "lzma"];
        for lib in libs {
            println!("cargo:rustc-link-lib={}", lib);
        }
    } else if target_os == "windows" && use_snmalloc {
        // snmalloc
        println!("cargo:rustc-link-arg=/nodefaultlib:libucrt.lib");

        let libs = ["kernel32", "ntdll", "shell32", "user32", "advapi32", "ucrt"];
        for lib in libs {
            println!("cargo:rustc-link-lib={}", lib);
        }
    } else if target_os == "linux" && use_snmalloc && target_env == "musl" {
        println!("cargo:rustc-link-lib=static=snmallocshim-rust");
    } else if target_os == "linux" && use_snmalloc {
        println!("cargo:rustc-link-lib=static=snmallocshim-rust");
        println!("cargo:rustc-link-lib=stdc++");
    } else if target_os == "windows" {
        // https://github.com/microsoft/mimalloc/blob/af21001f7a65eafb8fb16460b018ebf9d75e2ad8/CMakeLists.txt#L487
        let libs = ["psapi", "shell32", "user32", "advapi32", "bcrypt"];
        for lib in libs {
            println!("cargo:rustc-link-lib={}", lib);
        }
    } else if target_os == "linux" && use_mimalloc {
        let libs = ["atomic"];
        for lib in libs {
            println!("cargo:rustc-link-lib={}", lib);
        }
    }
}
