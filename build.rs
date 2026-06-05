use std::{path::Path, process::Command};

fn main() {
    println!("cargo:rerun-if-changed=src/object_out/linker_c/lld.cpp");

    let libdir = Command::new("llvm-config")
        .arg("--libdir")
        .output()
        .expect("Failed to run llvm-config");
    let libdir_str = String::from_utf8_lossy(&libdir.stdout).trim().to_string();

    let include_dir = Command::new("llvm-config")
        .arg("--includedir")
        .output()
        .expect("Failed to get LLVM includedir flags");

    println!("cargo:rustc-link-search=native={}", libdir_str);
    println!("cargo:rustc-link-lib=lldCommon");
    println!("cargo:rustc-link-lib=lldCOFF");
    println!("cargo:rustc-link-lib=lldELF");
    println!("cargo:rustc-link-lib=lldMachO");
    println!("cargo:rustc-link-lib=lldMinGW");
    println!("cargo:rustc-link-lib=lldWasm");
    println!("cargo:rustc-link-lib=llvmSupport");

    cc::Build::new()
        .cpp(true)
        .file("src/object_out/linker_c/lld.cpp")
        .std("c++17")
        .include(Path::new(&String::from_utf8(include_dir.stdout).unwrap().trim().to_string()))
        .compile("lld_cpp");

    println!("cargo:rustc-link-lib=msvcrt");
}