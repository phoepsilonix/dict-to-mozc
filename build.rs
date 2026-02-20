fn main() {
    // === Target Information (for cargo build, zigbuild, cargo-xwin) ===
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let target_env = std::env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();
    // let target_arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();

    // === Feature Flags ===
    let use_snmalloc = std::env::var("CARGO_FEATURE_USE_SNMALLOC").is_ok()
        || std::env::var("CARGO_FEATURE_USE_SNMALLOC_CC").is_ok();
    let use_tcmalloc_static = std::env::var("CARGO_FEATURE_USE_TCMALLOC_STATIC").is_ok();
    let use_mimalloc = std::env::var("CARGO_FEATURE_USE_MIMALLOC").is_ok();

    // === Dispatch ===
    match target_os.as_str() {
        "linux" => handle_linux(&target_env, use_snmalloc, use_tcmalloc_static, use_mimalloc),
        "windows" => handle_windows(use_snmalloc),
        _ => {} // ignore other platforms
    }
}

// === Linux-specific linker options ===
fn handle_linux(
    target_env: &str,
    use_snmalloc: bool,
    use_tcmalloc_static: bool,
    use_mimalloc: bool,
) {
    if use_tcmalloc_static {
        link_tcmalloc_static();
    } else if use_snmalloc {
        link_snmalloc_linux(target_env);
    } else if use_mimalloc {
        link_mimalloc_linux();
    }
}

// === Windows-specific linker options ===
fn handle_windows(use_snmalloc: bool) {
    if use_snmalloc {
        link_snmalloc_windows();
    } else {
        link_mimalloc_windows();
    }
}

// === Linking patterns ===

fn link_tcmalloc_static() {
    // lzma.a (pacman-static)
    // println!("cargo:rustc-link-search=native={}", "/usr/lib/pacman/lib");
    // println!("cargo:rustc-link-lib=static=lzma");

    // libunwind.a (rust-musl or rust-aarch64-musl)
    // println!("cargo:rustc-link-search=native={}{}{}", "/usr/lib/rustlib/", target_arch, "-unknown-linux-musl/lib/self-contained/");
    // println!("cargo:rustc-link-lib=static=unwind");

    // Static link
    {
        let lib = "tcmalloc";
        println!("cargo:rustc-link-lib=static={}", lib);
    }
    for lib in ["stdc++", "unwind", "lzma"] {
        println!("cargo:rustc-link-lib={}", lib);
    }
}

fn link_snmalloc_linux(target_env: &str) {
    // snmalloc (for Linux)
    if target_env == "musl" {
        println!("cargo:rustc-link-lib=static=snmallocshim-rust");
        println!("cargo:rustc-link-lib=atomic");
        println!("cargo:rustc-link-lib=stdc++");
    } else {
//        println!("cargo:rustc-link-lib=static=snmallocshim-rust");
//        println!("cargo:rustc-link-lib=snmallocshim-rust");
    }
}

fn link_snmalloc_windows() {
    // snmalloc for Windows
    println!("cargo:rustc-link-arg=/nodefaultlib:libucrt.lib");
    for lib in ["kernel32", "ntdll", "shell32", "user32", "advapi32", "ucrt"] {
        println!("cargo:rustc-link-lib={}", lib);
    }
}

fn link_mimalloc_windows() {
    // Ref: https://github.com/microsoft/mimalloc/blob/af21001f7a65eafb8fb16460b018ebf9d75e2ad8/CMakeLists.txt#L487
    for lib in ["psapi", "shell32", "user32", "advapi32", "bcrypt"] {
        println!("cargo:rustc-link-lib={}", lib);
    }
}

fn link_mimalloc_linux() {
    //for lib in ["atomic"] {
    //    println!("cargo:rustc-link-lib={}", lib);
    //}
}
