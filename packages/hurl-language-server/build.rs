use chrono::Utc;
use std::env::consts::{ARCH, OS};
use std::process::Command;

#[cfg(debug_assertions)]
const BUILD_TYPE: &'static str = "debug";

#[cfg(not(debug_assertions))]
const BUILD_TYPE: &'static str = "release";

fn main() {
    //Create the version string
    let version_string = create_version_string();

    //Set the version string to be used by the build
    println!("cargo:rustc-env=VERSION_STRING={}", version_string);
}

/// Create the version string for the build.
///
/// # Returns
/// The version string for the build.
///
/// # Examples
/// ```
/// hurl-language-server 0.1.0 (main:8b2cdea+, debug build, windows [x86_64], Jan 28 2025, 20:50:58)
/// ```
fn create_version_string() -> String {
    format!(
        "{} {} ({}:{}{}, {} build, {} [{}], {})",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION"),
        get_branch_name(),
        get_commit_hash(),
        if is_working_tree_clean() { "" } else { "+" },
        BUILD_TYPE,
        OS,
        ARCH,
        Utc::now().format("%b %d %Y, %T")
    )
}

/// Get abbreviated git hash during build
///
/// # Returns
/// abbreviated commit hash
fn get_commit_hash() -> String {
    let output = Command::new("git")
        .arg("log")
        .arg("-1")
        .arg("--pretty=format:%H") // Full commit hash
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .unwrap();

    assert!(output.status.success());

    String::from_utf8_lossy(&output.stdout).to_string()
}

/// Get the git branch name during build
///
/// # Returns
/// The git branch name
fn get_branch_name() -> String {
    let output = Command::new("git")
        .arg("rev-parse")
        .arg("--abbrev-ref")
        .arg("HEAD")
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .unwrap();

    assert!(output.status.success());

    String::from_utf8_lossy(&output.stdout)
        .trim_end()
        .to_string()
}

/// Is git working tree clean during build
///
/// # Returns
/// true if no uncommitted code changes were made before the build
fn is_working_tree_clean() -> bool {
    let status = Command::new("git")
        .arg("diff")
        .arg("--quiet")
        .arg("--exit-code")
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .status()
        .unwrap();

    status.code().unwrap() == 0
}
