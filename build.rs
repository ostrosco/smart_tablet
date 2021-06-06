use std::process::Command;

fn main() {
    // Run `npm install` to install any npm dependencies if they haven't already.
    let status = if cfg!(target_os = "windows") {
        Command::new("cmd")
            .current_dir("./frontend")
            .args(&["/C", "npm install"])
            .status()
    } else {
        Command::new("npm")
            .current_dir("./frontend")
            .arg("install")
            .status()
    };
    let status = match status {
        Ok(status) => status,
        Err(e) => panic!("Failed to run npm install: {:?}", e),
    };
    assert!(
        status.success(),
        "npm install failed with status: {:?}",
        status.success()
    );

    // Run `npx webpack` to package up everything nicely for the Rust webserver.
    let status = if cfg!(target_os = "windows") {
        Command::new("cmd")
            .current_dir("./frontend")
            .args(&["/C", "npx webpack"])
            .status()
    } else {
        Command::new("npx")
            .current_dir("./frontend")
            .arg("webpack")
            .status()
    };
    let status = match status {
        Ok(status) => status,
        Err(e) => panic!("Failed to run npx webpack: {:?}", e),
    };
    assert!(
        status.success(),
        "npx webpack failed with status: {:?}",
        status.success()
    );

    println!("cargo:rerun-if-changed=./frontend/src");
}
