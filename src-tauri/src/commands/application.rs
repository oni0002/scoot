use crate::commands::domain::Command;
use std::path::Path;
use walkdir::WalkDir;

/// Scans the specified directory list for applications
pub async fn load(
    directories: &[String],
    extensions: &[String],
) -> Result<Vec<Command>, crate::error::AppError> {
    let directories_clone = directories.to_vec();
    let extensions = extensions
        .iter()
        .map(|e| e.to_lowercase())
        .collect::<Vec<_>>();

    // Heavy I/O processing is executed in a worker thread
    let commands = tokio::task::spawn_blocking(move || {
        let mut commands = Vec::new();

        for dir_path in directories_clone {
            let expanded_path = crate::system::expand_env_vars(&dir_path);
            let path = Path::new(&expanded_path);

            if !path.exists() {
                continue;
            }

            // Recursively scan
            for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
                let path = entry.path();

                // Only target specified extensions
                if let Some(extension) = path.extension() {
                    let ext_str = extension.to_string_lossy().to_lowercase();
                    if extensions.contains(&ext_str) {
                        if let Some(command) = create_command(path) {
                            commands.push(command);
                        }
                    }
                }
            }
        }

        commands
    })
    .await
    .map_err(|e| crate::error::AppError::System(format!("Failed to scan applications: {}", e)))?;

    Ok(commands)
}

/// Creates a Command object from a path
fn create_command(path: &Path) -> Option<Command> {
    let file_stem = path.file_stem()?.to_string_lossy().to_string();
    let full_path = path.to_string_lossy().to_string();

    // Get parent directory name (for description)
    let parent = path
        .parent()
        .and_then(|p| p.file_name())
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "Unknown".to_string());

    Some(Command {
        id: String::new(),
        name: file_stem,
        category: "file".to_string(),
        source: "application".to_string(),
        command: full_path,
        description: parent,
        prompt: None,
        working_dir: None,
        show_window: None,
    })
}
