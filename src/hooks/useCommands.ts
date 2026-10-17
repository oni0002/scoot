import { useState, useCallback, useRef } from 'react';
import { TauriAPI } from '../tauri';
import { Command } from '../types';
import { getErrorMessage } from '../error';
import { DIRECT_OPEN_ID } from '../constants';

/** Outcome of a command mutation; failures carry the backend's reason so the UI can show it. */
export type CommandResult = { ok: true } | { ok: false; message: string };

export function useCommands() {
    const [commands, setCommands] = useState<Command[]>([]);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState<string | null>(null);
    const isInitialLoad = useRef(true);

    const loadCommands = useCallback(async () => {
        try {
            if (isInitialLoad.current) setLoading(true);
            const loadedCommands = await TauriAPI.getAllCommands();
            setCommands(loadedCommands);
            setError(null);
        } catch (err) {
            const message = getErrorMessage(err);
            console.error('Failed to load commands:', message);
            setError(`Could not load commands: ${message}`);
        } finally {
            isInitialLoad.current = false;
            setLoading(false);
        }
    }, []);

    const mutate = useCallback(
        async (action: () => Promise<unknown>, what: string): Promise<CommandResult> => {
            try {
                await action();
                await loadCommands();
                return { ok: true };
            } catch (err) {
                const message = getErrorMessage(err);
                console.error(`Failed to ${what}:`, message);
                return { ok: false, message };
            }
        },
        [loadCommands],
    );

    const addCommand = useCallback(
        (command: Command) => mutate(() => TauriAPI.addCommand(command), 'add command'),
        [mutate],
    );

    const updateCommand = useCallback(
        (command: Command) => mutate(() => TauriAPI.updateCommand(command), 'update command'),
        [mutate],
    );

    const deleteCommand = useCallback(
        (id: string, name: string) =>
            mutate(() => TauriAPI.deleteCommand(id), `delete command "${name}"`),
        [mutate],
    );

    /** Resolves with keepWindowOpen; rejects with the backend's error when the command fails. */
    const executeCommand = useCallback(
        async (command: Command, args: string[] = []): Promise<boolean> => {
            const result =
                command.id === DIRECT_OPEN_ID
                    ? await TauriAPI.openTarget(command.command)
                    : await TauriAPI.executeCommand(command.id, args);
            return result.keepWindowOpen;
        },
        [],
    );

    return {
        commands,
        loading,
        error,
        setError,
        loadCommands,
        addCommand,
        updateCommand,
        deleteCommand,
        executeCommand,
    };
}
