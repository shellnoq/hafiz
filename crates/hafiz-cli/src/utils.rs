//! Utility functions for Hafiz CLI

use anyhow::Result;
use chrono::{DateTime, Utc};
use glob::Pattern;
use std::path::Path;

/// Format a datetime for display
pub fn format_datetime(dt: &DateTime<Utc>) -> String {
    dt.format("%Y-%m-%d %H:%M:%S").to_string()
}

/// Format size as human readable
pub fn format_size(bytes: i64, human_readable: bool) -> String {
    if human_readable {
        humansize::format_size(bytes as u64, humansize::BINARY)
    } else {
        bytes.to_string()
    }
}

/// Check if a path matches include/exclude patterns
pub fn matches_patterns(path: &str, include: Option<&str>, exclude: Option<&str>) -> Result<bool> {
    // If exclude pattern matches, skip
    if let Some(exclude_pattern) = exclude {
        let pattern = Pattern::new(exclude_pattern)?;
        if pattern.matches(path) {
            return Ok(false);
        }
    }

    // If include pattern specified, must match
    if let Some(include_pattern) = include {
        let pattern = Pattern::new(include_pattern)?;
        return Ok(pattern.matches(path));
    }

    // No patterns or only exclude that didn't match
    Ok(true)
}

/// Get content type from file extension
pub fn guess_content_type(path: &str) -> String {
    let ext = Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");

    match ext.to_lowercase().as_str() {
        // Text
        "txt" => "text/plain",
        "html" | "htm" => "text/html",
        "css" => "text/css",
        "js" => "application/javascript",
        "json" => "application/json",
        "xml" => "application/xml",
        "csv" => "text/csv",
        "md" => "text/markdown",
        "yaml" | "yml" => "application/x-yaml",

        // Images
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "svg" => "image/svg+xml",
        "ico" => "image/x-icon",
        "bmp" => "image/bmp",
        "tiff" | "tif" => "image/tiff",

        // Audio
        "mp3" => "audio/mpeg",
        "wav" => "audio/wav",
        "ogg" => "audio/ogg",
        "flac" => "audio/flac",
        "m4a" => "audio/mp4",

        // Video
        "mp4" => "video/mp4",
        "webm" => "video/webm",
        "avi" => "video/x-msvideo",
        "mov" => "video/quicktime",
        "mkv" => "video/x-matroska",

        // Documents
        "pdf" => "application/pdf",
        "doc" => "application/msword",
        "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        "xls" => "application/vnd.ms-excel",
        "xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        "ppt" => "application/vnd.ms-powerpoint",
        "pptx" => "application/vnd.openxmlformats-officedocument.presentationml.presentation",

        // Archives
        "zip" => "application/zip",
        "tar" => "application/x-tar",
        "gz" | "gzip" => "application/gzip",
        "bz2" => "application/x-bzip2",
        "xz" => "application/x-xz",
        "7z" => "application/x-7z-compressed",
        "rar" => "application/vnd.rar",

        // Code
        "rs" => "text/x-rust",
        "py" => "text/x-python",
        "go" => "text/x-go",
        "java" => "text/x-java-source",
        "c" => "text/x-c",
        "cpp" | "cc" | "cxx" => "text/x-c++src",
        "h" | "hpp" => "text/x-c++hdr",
        "sh" => "application/x-sh",

        // Fonts
        "woff" => "font/woff",
        "woff2" => "font/woff2",
        "ttf" => "font/ttf",
        "otf" => "font/otf",
        "eot" => "application/vnd.ms-fontobject",

        // Other
        "wasm" => "application/wasm",
        "bin" | "exe" | "dll" | "so" => "application/octet-stream",

        _ => "application/octet-stream",
    }
    .to_string()
}

/// Convert storage class string to display format
pub fn format_storage_class(storage_class: Option<&str>) -> &str {
    storage_class.unwrap_or("STANDARD")
}

/// Confirm an action with the user
pub fn confirm(message: &str) -> bool {
    use std::io::{self, Write};

    print!("{} [y/N]: ", message);
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();

    matches!(input.trim().to_lowercase().as_str(), "y" | "yes")
}

/// Extract filename from a path or key
pub fn extract_filename(path: &str) -> &str {
    path.rsplit('/').next().unwrap_or(path)
}

/// Join path components (handling trailing slashes)
pub fn join_key(prefix: &str, name: &str) -> String {
    if prefix.is_empty() {
        name.to_string()
    } else if prefix.ends_with('/') {
        format!("{}{}", prefix, name)
    } else {
        format!("{}/{}", prefix, name)
    }
}

/// Determine destination key when copying a file to S3
pub fn determine_dest_key(
    source_path: &str,
    dest_key: Option<&str>,
    dest_is_prefix: bool,
) -> String {
    let filename = extract_filename(source_path);

    match dest_key {
        Some(key) if dest_is_prefix || key.ends_with('/') => join_key(key, filename),
        Some(key) => key.to_string(),
        None => filename.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matches_patterns() {
        // No patterns
        assert!(matches_patterns("file.txt", None, None).unwrap());

        // Include only
        assert!(matches_patterns("file.txt", Some("*.txt"), None).unwrap());
        assert!(!matches_patterns("file.log", Some("*.txt"), None).unwrap());

        // Exclude only
        assert!(matches_patterns("file.txt", None, Some("*.log")).unwrap());
        assert!(!matches_patterns("file.log", None, Some("*.log")).unwrap());

        // Both
        assert!(matches_patterns("file.txt", Some("*.txt"), Some("*.log")).unwrap());
        assert!(!matches_patterns("file.log", Some("*.txt"), Some("*.log")).unwrap());
    }

    #[test]
    fn test_guess_content_type() {
        assert_eq!(guess_content_type("file.txt"), "text/plain");
        assert_eq!(guess_content_type("image.png"), "image/png");
        assert_eq!(guess_content_type("app.json"), "application/json");
        assert_eq!(
            guess_content_type("unknown.xyz"),
            "application/octet-stream"
        );
    }

    #[test]
    fn test_extract_filename() {
        assert_eq!(extract_filename("path/to/file.txt"), "file.txt");
        assert_eq!(extract_filename("file.txt"), "file.txt");
        assert_eq!(extract_filename("/file.txt"), "file.txt");
    }

    #[test]
    fn test_join_key() {
        assert_eq!(join_key("", "file.txt"), "file.txt");
        assert_eq!(join_key("prefix", "file.txt"), "prefix/file.txt");
        assert_eq!(join_key("prefix/", "file.txt"), "prefix/file.txt");
    }

    #[test]
    fn test_determine_dest_key() {
        assert_eq!(
            determine_dest_key("/path/file.txt", Some("dest/"), true),
            "dest/file.txt"
        );
        assert_eq!(
            determine_dest_key("/path/file.txt", Some("dest/newname.txt"), false),
            "dest/newname.txt"
        );
        assert_eq!(
            determine_dest_key("/path/file.txt", None, false),
            "file.txt"
        );
    }
}
