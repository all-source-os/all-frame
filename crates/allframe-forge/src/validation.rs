//! Project name validation
//!
//! Validates that project names follow Rust package naming conventions.

use anyhow::Result;

/// Validate project name (must be valid Rust identifier)
///
/// # Rules
/// - No spaces allowed
/// - Cannot start with a number
/// - Only alphanumeric characters, underscores, and hyphens allowed
///
/// # Examples
/// ```
/// # use allframe_forge::validation::validate_project_name;
/// assert!(validate_project_name("my-project").is_ok());
/// assert!(validate_project_name("my_project").is_ok());
/// assert!(validate_project_name("myproject123").is_ok());
///
/// assert!(validate_project_name("my project").is_err());
/// assert!(validate_project_name("123project").is_err());
/// assert!(validate_project_name("my@project").is_err());
/// ```
pub fn validate_project_name(name: &str) -> Result<()> {
    // Check for spaces
    if name.contains(' ') {
        anyhow::bail!("Invalid project name: project names cannot contain spaces");
    }

    // Check if it starts with a number
    if name.chars().next().is_some_and(|c| c.is_ascii_digit()) {
        anyhow::bail!("Invalid project name: project names cannot start with a number");
    }

    // Check if it's a valid Rust identifier (alphanumeric + underscore + hyphen)
    if !name
        .chars()
        .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
    {
        anyhow::bail!(
            "Invalid project name: only alphanumeric characters, underscores, and hyphens are \
             allowed"
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_names() {
        assert!(validate_project_name("my-project").is_ok());
        assert!(validate_project_name("my_project").is_ok());
        assert!(validate_project_name("myproject").is_ok());
        assert!(validate_project_name("my-project-123").is_ok());
    }

    #[test]
    fn test_invalid_spaces() {
        let result = validate_project_name("my project");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("spaces"));
    }

    #[test]
    fn test_invalid_starts_with_number() {
        let result = validate_project_name("123project");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("number"));
    }

    #[test]
    fn test_invalid_special_chars() {
        assert!(validate_project_name("my@project").is_err());
        assert!(validate_project_name("my$project").is_err());
        assert!(validate_project_name("my project!").is_err());
    }
}
