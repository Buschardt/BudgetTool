use crate::core::error::AppError;

/// Normalize a user-supplied journal name: append `.journal` if no extension,
/// keep it if already `.journal`, or return an error for any other extension.
pub fn normalize_journal_name(name: &str) -> Result<String, AppError> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(AppError::BadRequest("journal name cannot be empty".into()));
    }
    match file_extension(trimmed) {
        None => Ok(format!("{trimmed}.journal")),
        Some("journal") => Ok(trimmed.to_string()),
        Some(ext) => Err(AppError::BadRequest(format!(
            "expected a .journal name but got '.{ext}'; omit the extension or use '.journal'"
        ))),
    }
}

/// Sanitize an uploaded filename: keep only the final path component, reject traversal.
/// Returns None if the result is empty or a bare dot-segment.
pub fn sanitize_filename(name: &str) -> Option<String> {
    // Strip path separators and take only the last segment
    let basename = name
        .replace('\\', "/")
        .split('/')
        .rfind(|s| !s.is_empty())
        .map(|s| s.to_string())?;

    // Reject dot-only segments and names containing null bytes
    if basename == "." || basename == ".." || basename.contains('\0') {
        return None;
    }

    Some(basename)
}

/// Extract the file extension from a filename.
/// Returns None for dotfiles (e.g. `.hidden`) and files with no extension.
pub fn file_extension(filename: &str) -> Option<&str> {
    // Find the last dot that isn't the very first character
    let dot_pos = filename[1..].rfind('.')?.checked_add(1)?;
    let ext = &filename[dot_pos + 1..];
    if ext.is_empty() { None } else { Some(ext) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_normal_filename() {
        assert_eq!(
            sanitize_filename("transactions.csv"),
            Some("transactions.csv".into())
        );
    }

    #[test]
    fn sanitize_strips_path_separators() {
        assert_eq!(sanitize_filename("../../etc/passwd"), Some("passwd".into()));
        assert_eq!(
            sanitize_filename("foo/bar/baz.journal"),
            Some("baz.journal".into())
        );
    }

    #[test]
    fn sanitize_strips_windows_separators() {
        assert_eq!(
            sanitize_filename("C:\\Users\\bob\\file.csv"),
            Some("file.csv".into())
        );
    }

    #[test]
    fn sanitize_rejects_bare_dotdot() {
        assert_eq!(sanitize_filename(".."), None);
        assert_eq!(sanitize_filename("."), None);
    }

    #[test]
    fn sanitize_rejects_null_byte() {
        assert_eq!(sanitize_filename("file\0name.csv"), None);
    }

    #[test]
    fn sanitize_rejects_empty() {
        assert_eq!(sanitize_filename(""), None);
        assert_eq!(sanitize_filename("///"), None);
    }

    #[test]
    fn normalize_adds_extension() {
        assert_eq!(normalize_journal_name("2026").unwrap(), "2026.journal");
    }

    #[test]
    fn normalize_keeps_journal_extension() {
        assert_eq!(
            normalize_journal_name("2026.journal").unwrap(),
            "2026.journal"
        );
    }

    #[test]
    fn normalize_rejects_other_extensions() {
        assert!(matches!(
            normalize_journal_name("foo.csv"),
            Err(AppError::BadRequest(_))
        ));
    }

    #[test]
    fn normalize_rejects_empty() {
        assert!(matches!(
            normalize_journal_name(""),
            Err(AppError::BadRequest(_))
        ));
        assert!(matches!(
            normalize_journal_name("   "),
            Err(AppError::BadRequest(_))
        ));
    }

    #[test]
    fn extension_detection() {
        assert_eq!(file_extension("foo.csv"), Some("csv"));
        assert_eq!(file_extension("foo.journal"), Some("journal"));
        assert_eq!(file_extension("foo.rules"), Some("rules"));
        assert_eq!(file_extension("noext"), None);
        assert_eq!(file_extension(".hidden"), None);
    }
}
