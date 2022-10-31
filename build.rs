use std::env;
use std::fs;
use std::process::Command;
use std::str;

fn rerun_if_changed(path: impl AsRef<std::path::Path>) {
    let path = path.as_ref().display().to_string();

    println!("cargo:rerun-if-changed={}", path);

    match std::fs::File::open(&path) {
        Ok(_f) => {}
        Err(e) => {
            println!("cargo:warning=\"{path}\" {e} - 'cargo:rerun-if-changed' is tracking a file but we can't open it!");
        }
    }
}

fn main() {
    build_shaders();
}

fn build_shaders() {
    println!("Building Shaders");

    for entry in fs::read_dir("shaders/").unwrap() {
        // ".flatten()" to skip over Err results is super unintuitive... so no thanks.
        #![allow(clippy::manual_flatten)]
        if let Ok(e) = entry {
            if e.file_type().unwrap().is_file() {
                let p = e.path().canonicalize().unwrap();
                rerun_if_changed(p);
            }
        }
    }

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
        panic!("shaders/build.sh failed");
    }

    println!("Building Shaders ✅");
}
