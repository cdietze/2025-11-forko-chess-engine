#[cfg(test)]
pub fn assert_eq_unordered<T: Ord + std::fmt::Debug + Clone>(a: &[T], b: &[T]) {
    let mut a = a.to_vec();
    let mut b = b.to_vec();
    a.sort();
    b.sort();
    assert_eq!(a, b);
}

pub fn with_separator(n: i32) -> String {
    let s = n.to_string();
    s.as_bytes()
        .rchunks(3)
        .rev()
        .map(std::str::from_utf8)
        .collect::<Result<Vec<_>, _>>()
        .unwrap()
        .join("_")
}

/// Build a human‑readable version descriptor from build metadata.
///
/// Order of preference:
/// - BUILD_GIT_TAG and/or BUILD_GIT_COMMIT if provided by build.rs
/// - CARGO_PKG_VERSION as a fallback
/// - empty string if nothing is available
pub fn build_version_descriptor() -> String {
    let mut parts: Vec<String> = Vec::new();

    if let Some(tag) = option_env!("BUILD_GIT_TAG") {
        if !tag.is_empty() {
            parts.push(format!("tag {}", tag));
        }
    }
    if let Some(commit) = option_env!("BUILD_GIT_COMMIT") {
        if !commit.is_empty() {
            parts.push(format!("commit {}", commit));
        }
    }

    if parts.is_empty() {
        if let Some(pkg_ver) = option_env!("CARGO_PKG_VERSION") {
            if !pkg_ver.is_empty() {
                parts.push(pkg_ver.to_string());
            }
        }
    }

    parts.join(", ")
}

/// Compose a display name for the engine, including version info if available.
pub fn engine_display_name() -> String {
    let version = build_version_descriptor();
    format!("Forko chess engine ({})", version)
}
