use regex::Regex;
use std::env;

/// Expand Windows environment variables like %APPDATA%
pub fn expand_env_vars(path: &str) -> String {
    let re = Regex::new(r"%([^%]+)%").unwrap();

    re.replace_all(path, |caps: &regex::Captures| {
        let key = &caps[1];
        env::var(key).unwrap_or_else(|_| format!("%{}%", key))
    })
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expand_existing_var() {
        let key = "SCOOT_TEST_VAR";
        let value = "test_value";
        env::set_var(key, value);

        let input = format!("path/to/%{}%/file", key);
        let expected = format!("path/to/{}/file", value);

        assert_eq!(expand_env_vars(&input), expected);

        env::remove_var(key);
    }

    #[test]
    fn test_expand_non_existing_var() {
        let input = "path/to/%NON_EXISTING_VAR%/file";
        assert_eq!(expand_env_vars(input), input);
    }

    #[test]
    fn test_no_vars() {
        let input = "path/to/file";
        assert_eq!(expand_env_vars(input), input);
    }
}
