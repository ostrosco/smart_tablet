use std::process::Command;

fn main() {
    let status = match Command::new("npm")
        .current_dir("./frontend")
        .arg("install")
        .status()
    {
        Ok(status) => status,
        Err(e) => panic!("Failed to run npm install: {:?}", e),
    };
    assert!(status.success(), "npm install failed with status: {:?}", status.success());
    let status = match Command::new("npx")
        .current_dir("./frontend")
        .arg("webpack")
        .status()
    {
        Ok(status) => status,
        Err(e) => panic!("Failed to run npx webpack: {:?}", e),
    };
    assert!(status.success(), "npx webpack failed with status: {:?}", status.success());
    println!("cargo:rerun-if-changed=./frontend/src");
}
