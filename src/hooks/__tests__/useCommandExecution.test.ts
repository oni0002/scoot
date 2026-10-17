import { renderHook, act } from '@testing-library/react';
import { useCommandExecution } from '../useCommandExecution';
import { TauriAPI } from '../../tauri';
import { Command } from '../../types';

jest.mock('../../tauri');

const command: Command = {
    id: 'u1',
    name: 'Missing report',
    category: 'file',
    source: 'user',
    command: 'C:\nope.pdf',
    description: '',
};

const setup = (executeCommand: jest.Mock) => {
    const resetState = jest.fn();
    const { result } = renderHook(() =>
        useCommandExecution({
            query: 'missing',
            results: [{ command, score: 0, matches: [] }],
            selectedIndex: 0,
            argumentMode: null,
            setQuery: jest.fn(),
            setSearchMode: jest.fn(),
            resetState,
            executeCommand,
        }),
    );
    return { result, resetState };
};

describe('useCommandExecution', () => {
    beforeEach(() => jest.mocked(TauriAPI.hideWindow).mockReset());

    it('clears the search after a successful run', async () => {
        const { result, resetState } = setup(jest.fn().mockResolvedValue(true));
        await act(() => result.current.runSelectedCommand());
        expect(resetState).toHaveBeenCalled();
    });

    it('keeps the query when the run failed, so it can be corrected', async () => {
        const { result, resetState } = setup(jest.fn().mockResolvedValue(null));
        await act(() => result.current.runSelectedCommand());
        expect(resetState).not.toHaveBeenCalled();
        expect(TauriAPI.hideWindow).not.toHaveBeenCalled();
    });

    it('hides the window when the command asks for it', async () => {
        const { result } = setup(jest.fn().mockResolvedValue(false));
        await act(() => result.current.runSelectedCommand());
        expect(TauriAPI.hideWindow).toHaveBeenCalled();
    });
});
