use std::fs;
use std::path::PathBuf;

pub const MANIFEST: &str = include_str!("../plugin/manifest.json");
pub const CODE_JS: &str = include_str!("../plugin/dist/code.js");
pub const UI_HTML: &str = include_str!("../plugin/dist/index.html");

/// Returns `~/.figma-mcp-rs/plugin` (or `%USERPROFILE%\.figma-mcp-rs\plugin` on Windows).
pub fn install_dir() -> Option<PathBuf> {
    let home = std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)?;
    Some(home.join(".figma-mcp-rs").join("plugin"))
}

/// Writes manifest.json, dist/code.js, dist/index.html and `.version` to `install_dir()`.
/// Skips all writes when `.version` already equals CARGO_PKG_VERSION and the
/// three files exist. Returns the manifest path, or an error message.
pub fn install() -> Result<PathBuf, String> {
    let base_dir = install_dir().ok_or_else(|| "no HOME/USERPROFILE in environment".to_string())?;
    let dist_dir = base_dir.join("dist");
    let manifest_path = base_dir.join("manifest.json");
    let code_js_path = dist_dir.join("code.js");
    let ui_html_path = dist_dir.join("index.html");
    let version_path = base_dir.join(".version");

    let pkg_version = env!("CARGO_PKG_VERSION");

    // Check if we can skip writing: version file matches and all files exist.
    if let Ok(existing_version) = fs::read_to_string(&version_path) {
        if existing_version.trim() == pkg_version
            && manifest_path.exists()
            && code_js_path.exists()
            && ui_html_path.exists()
        {
            return Ok(manifest_path);
        }
    }

    // Write order: directory, files, version marker last.
    fs::create_dir_all(&dist_dir)
        .map_err(|e| format!("failed to create dist directory: {e}"))?;

    fs::write(&manifest_path, MANIFEST)
        .map_err(|e| format!("failed to write manifest.json: {e}"))?;
    fs::write(&code_js_path, CODE_JS)
        .map_err(|e| format!("failed to write dist/code.js: {e}"))?;
    fs::write(&ui_html_path, UI_HTML)
        .map_err(|e| format!("failed to write dist/index.html: {e}"))?;

    // Write version marker last so a crash mid-write leaves a stale marker.
    fs::write(&version_path, pkg_version)
        .map_err(|e| format!("failed to write .version: {e}"))?;

    Ok(manifest_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn install_writes_and_then_skips() {
        let temp_dir = tempfile::tempdir().expect("failed to create temp dir");
        let temp_path = temp_dir.path().to_string_lossy().to_string();
        let old_home = std::env::var("HOME").ok();

        std::env::set_var("HOME", &temp_path);

        // First call: write
        let result1 = install();
        assert!(result1.is_ok());
        let manifest_path = result1.unwrap();

        // Verify files exist with non-empty contents.
        assert!(manifest_path.exists());
        let manifest_content = fs::read_to_string(&manifest_path).expect("failed to read manifest");
        assert!(!manifest_content.is_empty());

        let dist_dir = manifest_path.parent().unwrap().join("dist");
        let code_js_path = dist_dir.join("code.js");
        let ui_html_path = dist_dir.join("index.html");
        assert!(code_js_path.exists());
        assert!(ui_html_path.exists());

        let code_js_content = fs::read_to_string(&code_js_path).expect("failed to read code.js");
        assert!(!code_js_content.is_empty());
        let ui_html_content =
            fs::read_to_string(&ui_html_path).expect("failed to read index.html");
        assert!(!ui_html_content.is_empty());

        let version_path = manifest_path.parent().unwrap().join(".version");
        let version_content = fs::read_to_string(&version_path).expect("failed to read .version");
        assert_eq!(version_content.trim(), env!("CARGO_PKG_VERSION"));

        // Record modification time of code.js.
        let metadata1 = fs::metadata(&code_js_path).expect("failed to get metadata");
        let modified1 = metadata1
            .modified()
            .expect("failed to get modified time");

        // Sleep briefly to ensure time difference is detectable if a write occurred.
        std::thread::sleep(std::time::Duration::from_millis(10));

        // Second call: should skip writing.
        let result2 = install();
        assert!(result2.is_ok());

        // Verify modification time is unchanged.
        let metadata2 = fs::metadata(&code_js_path).expect("failed to get metadata");
        let modified2 = metadata2
            .modified()
            .expect("failed to get modified time");
        assert_eq!(modified1, modified2, "code.js was modified on second call");

        // Restore HOME.
        if let Some(old) = old_home {
            std::env::set_var("HOME", old);
        } else {
            std::env::remove_var("HOME");
        }
    }
}
