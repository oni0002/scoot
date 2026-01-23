/// %APPDATA% などのWindows環境変数を展開する
pub fn expand_env_vars(path: &str) -> String {
    let mut result = path.to_string();
    if result.contains('%') {
        for (key, value) in std::env::vars() {
            let placeholder = format!("%{}%", key);
            if result.contains(&placeholder) {
                result = result.replace(&placeholder, &value);
            }
        }
    }
    result
}
