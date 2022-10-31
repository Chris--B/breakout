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
    build_swift();
}

fn build_swift() {
    println!("Building Swift code");

    // See: https://haim.dev/posts/2020-09-10-linking-swift-code-into-rust-app/
    let vendor = env::var("CARGO_CFG_TARGET_VENDOR").unwrap();
    if vendor != "apple" {
        return;
    }

    let profile = env::var("PROFILE").unwrap();
    let arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    // let os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    let os = "macosx";
    let target_triple = format!("{arch}-{vendor}-{os}");

    let output = Command::new("swift")
        .args(&["build", "-c", &profile, "--arch", &arch])
        .current_dir("./Apple")
        .output()
        .expect("Failed to build Swift code");

    let stdout = str::from_utf8(&output.stdout).unwrap_or_default();
    eprintln!("stdout:\n{stdout}");
    eprintln!();

    let stderr = str::from_utf8(&output.stderr).unwrap_or_default();
    eprintln!("stderr:\n{stderr}");
    eprintln!();

    if !output.status.success() {
        panic!("'swift build' failed");
    }

    let cwd = std::env::current_dir().unwrap();
    let cwd = cwd.display();

    // Source files
    for entry in fs::read_dir("Apple/").unwrap() {
        // ".flatten()" to skip over Err results is super unintuitive... so no thanks.
        #![allow(clippy::manual_flatten)]
        if let Ok(e) = entry {
            if e.file_type().unwrap().is_file() {
                let p = e.path().canonicalize().unwrap();
                rerun_if_changed(p);
            }
        }
    }

    // Linker
    println!("cargo:rustc-link-search=native={cwd}/Apple/.build/{target_triple}/{profile}");
    println!("cargo:rustc-link-lib=static=RooibosPlatform");

    // `$ swift -print-target-info`
    // TODO: Query this dynamically instead
    let runtime_library_paths = &[
        "/Applications/Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/lib/swift/macosx",
        "/usr/lib/swift",
    ];
    for path in runtime_library_paths {
        println!("cargo:rustc-link-search=native={path}");
    }

    println!("Building Swift code ✅");
}
