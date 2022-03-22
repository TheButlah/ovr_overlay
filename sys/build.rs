fn main() {
    // include path openvr/headers
    let path = std::path::PathBuf::from(relative("/openvr/headers"));
    // This assumes all your C++ bindings are in main.rs
    let mut b = autocxx_build::Builder::new(relative("/src/lib.rs"), &[&path]).expect_build();
    // arbitrary library name, pick anything
    b.flag_if_supported("-std=c++14")
        .define("OPENVR_BUILD_STATIC", None)
        .compile("autocxx-demo");

    println!("cargo:rerun-if-changed={}", relative("/src/lib.rs"));
    println!("cargo:rerun-if-changed={}", relative("/openvr/lib/"));

    // Link the C++ libraries
    #[cfg(target_os = "windows")]
    {
        // Maybe we need these???
        println!("cargo:rustc-link-lib=shell32");
        println!("cargo:rustc-link-lib=kernel32");
        println!("cargo:rustc-link-lib=user32");

        println!(
            "cargo:rustc-link-search=native={}",
            relative("\\openvr\\lib\\win64")
        );
    }
    println!("cargo:rustc-link-lib=static=openvr_api");

    #[cfg(not(target_os = "windows"))]
    panic!("Haven't tested on platforms other than windows!");
}

fn relative(s: &str) -> String {
    format!("{}{}", env!("CARGO_MANIFEST_DIR"), s)
}
