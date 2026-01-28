use crate::domain::command::Command;

/// Scootの組み込みコマンド定義を取得
pub fn get_scoot_commands() -> Vec<Command> {
    use crate::domain::command::*;

    vec![
        Command {
            id: "scoot-add-command".to_string(),
            name: "Add Command".to_string(),
            category: "scoot".to_string(),
            command: CMD_SCOOT_ADD_COMMAND.to_string(),
            description: "Add a new command to the launcher".to_string(),
            prompt: None,
            working_dir: None,
            is_editable: false,
        },
        Command {
            id: "scoot-open-commands".to_string(),
            name: "Open Commands.json".to_string(),
            category: "scoot".to_string(),
            command: CMD_SCOOT_OPEN_COMMANDS.to_string(),
            description: "Open commands.json configuration file".to_string(),
            prompt: None,
            working_dir: None,
            is_editable: false,
        },
        Command {
            id: "scoot-open-config".to_string(),
            name: "Open Config.json".to_string(),
            category: "scoot".to_string(),
            command: CMD_SCOOT_OPEN_CONFIG.to_string(),
            description: "Open config.json configuration file".to_string(),
            prompt: None,
            working_dir: None,
            is_editable: false,
        },
        Command {
            id: "scoot-open-readme".to_string(),
            name: "Open README".to_string(),
            category: "scoot".to_string(),
            command: CMD_SCOOT_OPEN_README.to_string(),
            description: "Open application README".to_string(),
            prompt: None,
            working_dir: None,
            is_editable: false,
        },
        Command {
            id: "scoot-open-log".to_string(),
            name: "Open Logs".to_string(),
            category: "scoot".to_string(),
            command: CMD_SCOOT_OPEN_LOG.to_string(),
            description: "Open application log directory".to_string(),
            prompt: None,
            working_dir: None,
            is_editable: false,
        },
        Command {
            id: "scoot-reload".to_string(),
            name: "Reload".to_string(),
            category: "scoot".to_string(),
            command: CMD_SCOOT_RELOAD.to_string(),
            description: "Reload commands and configuration".to_string(),
            prompt: None,
            working_dir: None,
            is_editable: false,
        },
        Command {
            id: "scoot-kill".to_string(),
            name: "Kill Scoot".to_string(),
            category: "scoot".to_string(),
            command: CMD_SCOOT_KILL.to_string(),
            description: "Terminate the application".to_string(),
            prompt: None,
            working_dir: None,
            is_editable: false,
        },
    ]
}
