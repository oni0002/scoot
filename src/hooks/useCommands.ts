import { useState, useCallback } from 'react';
import { TauriAPI } from '../tauri';
import { Command } from '../types';
import { getErrorMessage } from '../error';

export function useCommands() {
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
            console.error('Failed to load commands:', getErrorMessage(err));
            setError('Failed to load commands');
        } finally {
            setLoading(false);
        }
    }, []);

    const addCommand = useCallback(async (command: Command) => {
        try {
            await TauriAPI.addCommand(command);
            await loadCommands();
            return true;
        } catch (err) {
            console.error('Failed to add command:', getErrorMessage(err));
            return false;
        }
    }, [loadCommands]);

    const updateCommand = useCallback(async (command: Command) => {
        try {
            await TauriAPI.updateCommand(command);
            await loadCommands();
            return true;
        } catch (err) {
            console.error('Failed to update command:', getErrorMessage(err));
            return false;
        }
    }, [loadCommands]);

    const deleteCommand = useCallback(async (id: string, name: string) => {
        try {
            await TauriAPI.deleteCommand(id);
            await loadCommands();
            return true;
        } catch (err) {
            console.error(`Failed to delete command "${name}":`, getErrorMessage(err));
            return false;
        }
    }, [loadCommands]);

    const executeCommand = useCallback(async (command: Command, args: string[] = []): Promise<boolean | null> => {
        try {
            const result = await TauriAPI.executeCommand(command, args);
            return result.keepWindowOpen;
        } catch (err) {
            console.error(`Failed to execute command "${command.name}":`, getErrorMessage(err));
            return null;
        }
    }, []);

    return {
        commands,
        loading,
        error,
        setError,
        loadCommands,
        addCommand,
        updateCommand,
        deleteCommand,
        executeCommand
    };
}
