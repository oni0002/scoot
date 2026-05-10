import { useState, useCallback } from 'react';
import { TauriAPI } from '../api/tauri';
import { Command } from '../types';
import { NOTIFICATION_DURATION } from '../constants';
import { getErrorMessage } from '../utils/error';



import { useNotificationContext } from '../context/NotificationContext';

export function useCommands() {
    const { showSuccess, showError, showInfo } = useNotificationContext();
    const [commands, setCommands] = useState<Command[]>([]);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState<string | null>(null);

    const loadCommands = useCallback(async () => {
        try {
            setLoading(true);
            const loadedCommands = await TauriAPI.getAllCommands();
            setCommands(loadedCommands);
            setError(null);
        } catch (err) {
            const errorMessage = getErrorMessage(err);
            console.error('Failed to load commands:', errorMessage);
            setError('Failed to load commands');
            showError("Could not load commands from configuration.", NOTIFICATION_DURATION.VERY_LONG);
        } finally {
            setLoading(false);
        }
    }, [showError]);

    const addCommand = useCallback(async (command: Command) => {
        try {
            await TauriAPI.addCommand(command);
            showSuccess(`"${command.name}" added successfully.`);
            await loadCommands();
            return true;
        } catch (err) {
            const errorMessage = getErrorMessage(err);
            console.error('Failed to add command:', errorMessage);
            showError(`Failed to add command: ${errorMessage}`, NOTIFICATION_DURATION.LONG);
            return false;
        }
    }, [loadCommands, showSuccess, showError]);

    const updateCommand = useCallback(async (command: Command) => {
        try {
            await TauriAPI.updateCommand(command);
            showSuccess(`"${command.name}" updated successfully.`);
            await loadCommands();
            return true;
        } catch (err) {
            const errorMessage = getErrorMessage(err);
            console.error('Failed to update command:', errorMessage);
            showError(`Failed to update command: ${errorMessage}`, NOTIFICATION_DURATION.LONG);
            return false;
        }
    }, [loadCommands, showSuccess, showError]);

    const deleteCommand = useCallback(async (id: string, name: string) => {
        try {
            await TauriAPI.deleteCommand(id);
            showSuccess(`"${name}" deleted successfully.`);
            await loadCommands();
            return true;
        } catch (err) {
            const errorMessage = getErrorMessage(err);
            console.error(`Failed to delete command "${name}":`, errorMessage);
            showError(`Could not delete "${name}".`, NOTIFICATION_DURATION.LONG);
            return false;
        }
    }, [loadCommands, showSuccess, showError]);

    const ignoreCommand = useCallback(async (command: Command) => {
        try {
            const config = await TauriAPI.getConfig();
            if (!config.ignored.includes(command.command)) {
                const updated = { ...config, ignored: [...config.ignored, command.command] };
                await TauriAPI.saveAppConfig(updated);
                await TauriAPI.reloadAll();
            }
            showSuccess(`"${command.name}" ignored successfully.`);
            await loadCommands();
            return true;
        } catch (err) {
            const errorMessage = getErrorMessage(err);
            console.error(`Failed to ignore command "${command.name}":`, errorMessage);
            showError(`Could not ignore "${command.name}".`, NOTIFICATION_DURATION.LONG);
            return false;
        }
    }, [loadCommands, showSuccess, showError]);

    const executeCommand = useCallback(async (command: Command, args: string[] = []) => {
        try {
            const result = await TauriAPI.executeCommand(command, args);
            return result.keepWindowOpen;
        } catch (err) {
            const errorMessage = getErrorMessage(err);
            console.error(`Failed to execute command "${command.name}":`, errorMessage);
            showError(`Failed to execute "${command.name}".`, NOTIFICATION_DURATION.LONG);
            return false;
        }
    }, [showInfo, showError]);

    return {
        commands,
        loading,
        error,
        setError,
        loadCommands,
        addCommand,
        updateCommand,
        deleteCommand,
        ignoreCommand,
        executeCommand
    };
}
