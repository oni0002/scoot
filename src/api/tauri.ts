import { invoke } from '@tauri-apps/api/core';
import { message } from '@tauri-apps/plugin-dialog';
import { getVersion } from '@tauri-apps/api/app';
import { Command, Config, Commands } from '../types';

export class TauriAPI {
  static async getAllCommands(): Promise<Commands> {
    return await invoke<Commands>('get_all_commands');
  }

  static async addCommand(command: Command): Promise<string> {
    return await invoke('add_command', { command });
  }

  static async updateCommand(command: Command): Promise<void> {
    return await invoke('update_command', { command });
  }

  static async deleteCommand(id: string): Promise<void> {
    return await invoke('delete_command', { id });
  }

  static async executeCommand(command: Command, args: string[] = []): Promise<string> {
    return await invoke('execute_command', { command, args });
  }

  static async getCommandsByPrompt(prompt: string): Promise<Command[]> {
    return await invoke('get_commands_by_prompt', { prompt });
  }

  static async reloadConfig(): Promise<void> {
    return await invoke('reload_config');
  }

  static async toggleWindow(): Promise<void> {
    return await invoke('toggle_window');
  }

  static async hideWindow(): Promise<void> {
    return await invoke('hide_window');
  }

  static async showWindow(): Promise<void> {
    return await invoke('show_window');
  }

  static async openCommandsJson(): Promise<void> {
    return await invoke('open_commands_json');
  }

  static async openConfigJson(): Promise<void> {
    return await invoke('open_config_json');
  }

  static async openReadme(): Promise<void> {
    return await invoke('open_readme');
  }

  static async quitApp(): Promise<void> {
    return await invoke('quit_app');
  }

  static async setPreventHide(prevent: boolean): Promise<void> {
    return await invoke('set_prevent_hide', { prevent });
  }

  static async getConfig(): Promise<Config> {
    return await invoke<Config>('get_config');
  }

  static async saveAppConfig(config: Config): Promise<void> {
    return await invoke('save_config', { config });
  }

  static async getCommandsConfig(): Promise<Commands> {
    return await invoke('get_commands');
  }

  static async saveCommandsConfig(commands: Commands): Promise<void> {
    return await invoke('save_commands', { commands });
  }

  static async getAppConfigFilePath(): Promise<string> {
    return await invoke('get_config_file_path');
  }

  static async getCommandsFilePath(): Promise<string> {
    return await invoke('get_commands_file_path');
  }

  // スキーマ関連メソッド
  static async getAppConfigSchema(): Promise<any> {
    return await invoke('get_config_schema');
  }

  static async getCommandsConfigSchema(): Promise<any> {
    return await invoke('get_commands_schema');
  }

  static async validateAppConfig(config: any): Promise<{ valid: boolean; errors: string[] }> {
    return await invoke('validate_config', { config });
  }

  static async validateCommandsConfig(config: any): Promise<{ valid: boolean; errors: string[] }> {
    return await invoke('validate_commands', { config });
  }

  static async getVersion(): Promise<string> {
    return await getVersion();
  }

  static async showMessage(msg: string, title: string = 'Scoot'): Promise<void> {
    await message(msg, { title, kind: 'info' });
  }

  static async ignoreCommand(commandPath: string): Promise<void> {
    await invoke('ignore_command', { commandPath });
  }
}
