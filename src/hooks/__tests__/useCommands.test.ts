import { act, renderHook } from '@testing-library/react';
import { invoke } from '@tauri-apps/api/core';
import { useCommands } from '../useCommands';
import { DIRECT_OPEN_ID } from '../../constants';
import { Command } from '../../types';

const mockInvoke = jest.mocked(invoke);

const command: Command = {
    id: 'u1',
    name: 'Docs',
    category: 'url',
    command: 'https://example.com',
    description: '',
};

beforeEach(() => {
    mockInvoke.mockReset();
});

describe('useCommands mutations', () => {
    it('reloads the list and reports success', async () => {
        mockInvoke.mockResolvedValueOnce('u2').mockResolvedValueOnce([command]);
        const { result } = renderHook(() => useCommands());

        let outcome: unknown;
        await act(async () => {
            outcome = await result.current.addCommand(command);
        });

        expect(outcome).toEqual({ ok: true });
        expect(mockInvoke).toHaveBeenLastCalledWith('get_all_commands');
        expect(result.current.commands).toEqual([command]);
    });

    it('returns the backend reason without its category prefix on failure', async () => {
        mockInvoke.mockRejectedValueOnce(
            "Validation error: Alias 'd' is already used by another command.",
        );
        const { result } = renderHook(() => useCommands());

        let outcome: unknown;
        await act(async () => {
            outcome = await result.current.addCommand(command);
        });

        expect(outcome).toEqual({
            ok: false,
            message: "Alias 'd' is already used by another command.",
        });
    });

    it('reports a failed delete', async () => {
        mockInvoke.mockRejectedValueOnce('Not found: Command not found');
        const { result } = renderHook(() => useCommands());

        let outcome: unknown;
        await act(async () => {
            outcome = await result.current.deleteCommand('u1', 'Docs');
        });

        expect(outcome).toEqual({ ok: false, message: 'Command not found' });
    });
});

describe('useCommands.executeCommand', () => {
    it('runs a registered command by id', async () => {
        mockInvoke.mockResolvedValueOnce({ keepWindowOpen: false });
        const { result } = renderHook(() => useCommands());

        await expect(result.current.executeCommand(command, ['a'])).resolves.toBe(false);
        expect(mockInvoke).toHaveBeenCalledWith('execute_command', { id: 'u1', args: ['a'] });
    });

    it('opens a typed URL or path through open_target', async () => {
        mockInvoke.mockResolvedValueOnce({ keepWindowOpen: false });
        const { result } = renderHook(() => useCommands());
        const direct: Command = { ...command, id: DIRECT_OPEN_ID, command: 'https://x.dev' };

        await result.current.executeCommand(direct);
        expect(mockInvoke).toHaveBeenCalledWith('open_target', { target: 'https://x.dev' });
    });

    it('rejects with the backend error so the caller can show the cause', async () => {
        mockInvoke.mockRejectedValueOnce("Command execution error: File not found: 'x'.");
        const { result } = renderHook(() => useCommands());

        await expect(result.current.executeCommand(command)).rejects.toBe(
            "Command execution error: File not found: 'x'.",
        );
    });
});
