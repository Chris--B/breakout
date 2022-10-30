use std::env;
use std::fs;
use std::process::Command;
use std::str;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    for entry in fs::read_dir("shaders/").unwrap() {
        // ".flatten()" to skip over Err results is super unintuitive... so no thanks.
        #![allow(clippy::manual_flatten)]
        if let Ok(e) = entry {
            if e.file_type().unwrap().is_file() {
                let p = e.path().canonicalize().unwrap();
                println!("cargo:rerun-if-changed={}", p.display());
            }
        }
    }

    // Compile shaders
    let output = Command::new("bash")
        .arg("shaders/build.sh")
        .arg(std::env::var("OUT_DIR").unwrap())
        .output()
        .expect("Failed to build shaders");

    let stdout = str::from_utf8(&output.stdout).unwrap_or_default();
    eprintln!("stdout:\n{stdout}");
    eprintln!();

    let stderr = str::from_utf8(&output.stderr).unwrap_or_default();
    eprintln!("stderr:\n{stderr}");
    eprintln!();

    if !output.status.success() {
        panic!("shaders/build.sh failed:");
    }

    build_swift();
}

fn build_swift() {
    let vendor = env::var("CARGO_CFG_TARGET_VENDOR").unwrap();
    if vendor != "apple" {
        return;
    }

    let profile = env::var("PROFILE").unwrap();
    let arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    // let os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    let os = "macosx";
    let target_triple = format!("{arch}-{vendor}-{os}");

    if !Command::new("swift")
        .args(&["build", "-c", &profile, "--arch", &arch])
        .current_dir("./src/swift")
        .status()
        .unwrap()
        .success()
    {
        panic!("Swift compilation failed")
    }

    println!("cargo:rustc-link-search=native=src/swift/.build/{target_triple}/{profile}");
    println!("cargo:rustc-link-lib=static=RooibosPlatform");
    println!("cargo:rerun-if-changed=src/swift/*.swift");

    // `$ swift -print-target-info`
    // TODO: Query this dynamically instead
    let runtime_library_paths = &[
        "/Applications/Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/lib/swift/macosx",
        "/usr/lib/swift",
    ];

    for path in runtime_library_paths {
        println!("cargo:rustc-link-search=native={path}");
    }
}
