use std::process::Command;

fn main() {
    let status = Command::new("npm")
        .args(&["--prefix", "./frontend", "install", "./frontend"])
        .status()
        .unwrap();
    assert!(status.success());
    let status = Command::new("npx")
        .current_dir("./frontend")
        .args(&["webpack"])
        .status()
        .unwrap();
    assert!(status.success());
    println!("cargo:rerun-if-changed=./frontend/src");
}
