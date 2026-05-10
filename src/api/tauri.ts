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

  static async executeCommand(command: Command, args: string[] = []): Promise<{ keepWindowOpen: boolean }> {
    return await invoke('execute_command', { command, args });
  }

  static async getCommandsByPrompt(prompt: string): Promise<Command[]> {
    return await invoke('get_commands_by_prompt', { prompt });
  }

  static async reloadAll(): Promise<void> {
    return await invoke('reload_all');
  }

  static async hideWindow(): Promise<void> {
    return await invoke('hide_window');
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

  static async enterModal(): Promise<void> {
    return await invoke('enter_modal');
  }

  static async leaveModal(): Promise<void> {
    return await invoke('leave_modal');
  }

  static async getConfig(): Promise<Config> {
    return await invoke<Config>('get_config');
  }

  static async saveAppConfig(config: Config): Promise<void> {
    return await invoke('save_config', { config });
  }

  static async getVersion(): Promise<string> {
    return await getVersion();
  }

  static async showMessage(msg: string, title: string = 'Scoot'): Promise<void> {
    await message(msg, { title, kind: 'info' });
  }

}
