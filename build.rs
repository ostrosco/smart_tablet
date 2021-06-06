use std::process::Command;

fn main() {
    let status = Command::new("npm")
        .args(&["--prefix", "./frontend", "install", "./frontend"])
        .status()
        .expect("Failed to run npm install.");
    assert!(status.success());
    let status = Command::new("npx")
        .current_dir("./frontend")
        .args(&["webpack"])
        .status()
        .expect("Failed to run npx webpack.");
    assert!(status.success());
    println!("cargo:rerun-if-changed=./frontend/src");
}
