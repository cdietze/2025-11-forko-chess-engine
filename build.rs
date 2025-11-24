use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn main() {
    // Attempt to get the latest git tag
    let git_tag = run(&["git", "describe", "--tags", "--abbrev=0"]) 
        .or_else(|| run(&["git", "describe", "--tags", "--always"]))
        .unwrap_or_default();

    // Attempt to get the current commit hash (short)
    let git_commit = run(&["git", "rev-parse", "--short=12", "HEAD"]).unwrap_or_default();

    println!("cargo:rustc-env=BUILD_GIT_TAG={}", git_tag);
    println!("cargo:rustc-env=BUILD_GIT_COMMIT={}", git_commit);

    // Re-run the build script when git HEAD changes or this script changes
    println!("cargo:rerun-if-changed=build.rs");
    // Best-effort triggers for common git state files; safe if missing.
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-changed=.git/refs/heads");
    println!("cargo:rerun-if-changed=.git/packed-refs");
}

fn run(cmd: &[&str]) -> Option<String> {
    if cmd.is_empty() { return None; }
    let (prog, args) = cmd.split_first().unwrap();
    let out = Command::new(prog).args(args).output().ok()?;
    if !out.status.success() { return None; }
    let s = String::from_utf8(out.stdout).ok()?.trim().to_string();
    if s.is_empty() { None } else { Some(s) }
}
