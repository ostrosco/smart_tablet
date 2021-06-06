use std::process::Command;

fn main() {
    let status = Command::new("yarn")
        .args(&["--cwd", "./frontend"])
        .status()
        .unwrap();
    assert!(status.success());
    let status = Command::new("yarn")
        .args(&["--cwd", "./frontend", "build"])
        .status()
        .unwrap();
    assert!(status.success());
    println!("cargo:rerun-if-changed=./frontend/src");
}
