use std::path::Path;

/// Maps a file's extension to a language ID. Add entries here as you
/// support more languages — this is the single source of truth.
pub fn language_for_path(path: &Path) -> Option<&'static str> {
    let ext = path.extension()?.to_str()?;
    Some(match ext {
        "rs" => "rust",
        "py" => "python",
        "ts" | "tsx" => "typescript",
        "js" | "jsx" => "javascript",
        "go" => "go",
        "c" | "h" => "c",
        "cpp" | "cc" | "cxx" | "hpp" => "cpp",
        _ => return None,
    })
}

/// Candidate server binaries, in fallback order, for a given language.
fn candidates_for(language: &str) -> &'static [&'static str] {
    match language {
        "rust" => &["rust-analyzer"],
        "python" => &["pyright-langserver", "pylsp"],
        "typescript" | "javascript" => &["typescript-language-server"],
        "go" => &["gopls"],
        "c" | "cpp" => &["clangd"],
        _ => &[],
    }
}

/// Some servers need a flag to speak LSP over stdio instead of defaulting
/// to some other transport. Missing this is a real, easy-to-hit bug —
/// don't skip it per-binary.
fn args_for(binary: &str) -> &'static [&'static str] {
    match binary {
        "pyright-langserver" => &["--stdio"],
        "typescript-language-server" => &["--stdio"],
        _ => &[],
    }
}

/// Marker file used to find a workspace root for a language, so e.g.
/// a Python project doesn't get rooted by an unrelated parent Cargo.toml.
fn root_marker_for(language: &str) -> &'static str {
    match language {
        "rust" => "Cargo.toml",
        "python" => "pyproject.toml",
        "typescript" | "javascript" => "package.json",
        "go" => "go.mod",
        "c" | "cpp" => "CMakeLists.txt",
        _ => "",
    }
}

/// Searches $PATH directly for the first available candidate binary.
/// No shell, no `which` dependency, no automatic installs.
pub fn find_server(language: &str) -> Option<(String, &'static [&'static str])> {
    let path_var = std::env::var_os("PATH")?;
    for candidate in candidates_for(language) {
        for dir in std::env::split_paths(&path_var) {
            let plain = dir.join(candidate);
            if plain.is_file() {
                return Some((candidate.to_string(), args_for(candidate)));
            }
            let exe = dir.join(format!("{candidate}.exe"));
            if exe.is_file() {
                return Some((candidate.to_string(), args_for(candidate)));
            }
        }
    }
    None
}

pub fn find_root(start: &Path, language: &str) -> std::path::PathBuf {
    let marker = root_marker_for(language);
    if marker.is_empty() {
        return start.to_path_buf();
    }
    start
        .ancestors()
        .find(|path| path.join(marker).is_file())
        .map(Path::to_path_buf)
        .unwrap_or_else(|| start.to_path_buf())
}
