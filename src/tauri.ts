import { invoke } from '@tauri-apps/api/core';
import { message } from '@tauri-apps/plugin-dialog';
import { getVersion } from '@tauri-apps/api/app';
import { Command, Config } from './types';

export const TauriAPI = {
  async getAllCommands(): Promise<Command[]> {
    return await invoke<Command[]>('get_all_commands');
  },

  async addCommand(command: Command): Promise<string> {
    return await invoke('add_command', { command });
  },

  async updateCommand(command: Command): Promise<void> {
    return await invoke('update_command', { command });
  },

  async deleteCommand(id: string): Promise<void> {
    return await invoke('delete_command', { id });
  },

  async executeCommand(command: Command, args: string[] = []): Promise<{ keepWindowOpen: boolean }> {
    return await invoke('execute_command', { command, args });
  },

  async getCommandsByPrompt(prompt: string): Promise<Command[]> {
    return await invoke('get_commands_by_prompt', { prompt });
  },

  async reloadAll(): Promise<void> {
    return await invoke('reload_all');
  },

  async hideWindow(): Promise<void> {
    return await invoke('hide_window');
  },

  async openCommandsJson(): Promise<void> {
    return await invoke('open_commands_json');
  },

  async openConfigJson(): Promise<void> {
    return await invoke('open_config_json');
  },

  async openReadme(): Promise<void> {
    return await invoke('open_readme');
  },

  async quitApp(): Promise<void> {
    return await invoke('quit_app');
  },

  async enterModal(): Promise<void> {
    return await invoke('enter_modal');
  },

  async leaveModal(): Promise<void> {
    return await invoke('leave_modal');
  },

  async getConfig(): Promise<Config> {
    return await invoke<Config>('get_config');
  },

  async saveConfig(config: Config): Promise<void> {
    return await invoke('save_config', { config });
  },

  async getVersion(): Promise<string> {
    return await getVersion();
  },

  async showMessage(msg: string, title: string = 'Scoot'): Promise<void> {
    await message(msg, { title, kind: 'info' });
  },
};
