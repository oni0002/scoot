import { useState, useCallback, useMemo, useRef } from 'react';
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

    // The backend only adds an id and the source, so a success is applied to the list in place
    // instead of fetching every command (bookmarks and applications included) again.
    const mutate = useCallback(
        async <T>(
            action: () => Promise<T>,
            apply: (list: Command[], result: T) => Command[],
            what: string,
        ): Promise<CommandResult> => {
            try {
                const result = await action();
                setCommands((list) => apply(list, result));
                return { ok: true };
            } catch (err) {
                const message = getErrorMessage(err);
                console.error(`Failed to ${what}:`, message);
                return { ok: false, message };
            }
        },
        [],
    );

    const addCommand = useCallback(
        (command: Command) =>
            mutate(
                () => TauriAPI.addCommand(command),
                (list, id) => {
                    const added: Command = { ...command, id, source: 'user' };
                    // User commands come first in the list, as the backend returns them: add after the last one
                    const at = list.filter((c) => c.source === 'user').length;
                    return [...list.slice(0, at), added, ...list.slice(at)];
                },
                'add command',
            ),
        [mutate],
    );

    const updateCommand = useCallback(
        (command: Command) =>
            mutate(
                () => TauriAPI.updateCommand(command),
                (list) =>
                    list.map((c) => (c.id === command.id ? { ...command, source: 'user' } : c)),
                'update command',
            ),
        [mutate],
    );

    const deleteCommand = useCallback(
        (id: string, name: string) =>
            mutate(
                () => TauriAPI.deleteCommand(id),
                (list) => list.filter((c) => c.id !== id),
                `delete command "${name}"`,
            ),
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

    // Stable while nothing changes, so context consumers do not re-render with every provider render
    return useMemo(
        () => ({
            commands,
            loading,
            error,
            setError,
            loadCommands,
            addCommand,
            updateCommand,
            deleteCommand,
            executeCommand,
        }),
        [
            commands,
            loading,
            error,
            loadCommands,
            addCommand,
            updateCommand,
            deleteCommand,
            executeCommand,
        ],
    );
}
