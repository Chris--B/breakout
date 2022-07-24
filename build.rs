use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    for entry in std::fs::read_dir("shaders/").unwrap() {
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

    let stdout = std::str::from_utf8(&output.stdout).unwrap_or_default();
    eprintln!("stdout:\n{stdout}");
    eprintln!();

    let stderr = std::str::from_utf8(&output.stderr).unwrap_or_default();
    eprintln!("stderr:\n{stderr}");
    eprintln!();

    if !output.status.success() {
        panic!("shaders/build.sh failed:");
    }
}
