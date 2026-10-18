import { invoke } from '@tauri-apps/api/core';
import { message, open } from '@tauri-apps/plugin-dialog';
import { getVersion } from '@tauri-apps/api/app';
import { Command, Settings, Notice, UsageMap } from './types';

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

    async executeCommand(id: string, args: string[] = []): Promise<{ keepWindowOpen: boolean }> {
        return await invoke('execute_command', { id, args });
    },

    async getUsage(): Promise<UsageMap> {
        return await invoke<UsageMap>('get_usage');
    },

    async clearUsage(): Promise<void> {
        return await invoke('clear_usage');
    },

    async openTarget(target: string): Promise<{ keepWindowOpen: boolean }> {
        return await invoke('open_target', { target });
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

    async openSettingsJson(): Promise<void> {
        return await invoke('open_config_json');
    },

    async enterModal(): Promise<void> {
        return await invoke('enter_modal');
    },

    async leaveModal(): Promise<void> {
        return await invoke('leave_modal');
    },

    /** The problems that still hold (a broken settings, a Hotkey another app holds). */
    async getNotices(): Promise<Notice[]> {
        return await invoke<Notice[]>('get_notices');
    },

    async getStartupShortcut(): Promise<boolean> {
        return await invoke<boolean>('get_startup_shortcut');
    },

    async setStartupShortcut(enabled: boolean): Promise<boolean> {
        return await invoke<boolean>('set_startup_shortcut', { enabled });
    },

    async startHotkeyCapture(): Promise<void> {
        return await invoke('start_hotkey_capture');
    },

    async stopHotkeyCapture(): Promise<void> {
        return await invoke('stop_hotkey_capture');
    },

    async getSettings(): Promise<Settings> {
        return await invoke<Settings>('get_settings');
    },

    async saveSettings(settings: Settings): Promise<void> {
        return await invoke('save_settings', { settings });
    },

    async ignoreCommand(commandStr: string): Promise<void> {
        return await invoke('ignore_command', { commandStr });
    },

    async unignoreCommand(commandStr: string): Promise<void> {
        return await invoke('unignore_command', { commandStr });
    },

    /** Sets the alias of an external command (bookmark, application, ...); null clears it. */
    async setExternalAlias(commandStr: string, alias: string | null): Promise<void> {
        return await invoke('set_external_alias', { commandStr, alias });
    },

    async getVersion(): Promise<string> {
        return await getVersion();
    },

    async showMessage(msg: string, title: string = 'Scoot'): Promise<void> {
        await message(msg, { title, kind: 'info' });
    },

    /** Asks for a file or a folder; null when the user cancels. */
    async pickPath(directory: boolean): Promise<string | null> {
        const selected = await open({ multiple: false, directory });
        return typeof selected === 'string' ? selected : null;
    },
};
