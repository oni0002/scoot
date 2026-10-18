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

describe('useCommandExecution in argument mode', () => {
    const search: Command = {
        ...command,
        id: 'u2',
        name: 'Search',
        category: 'url',
        command: 'https://example.com/?q={$1}',
        alias: 's',
    };
    const setupArguments = (query: string, executeCommand: jest.Mock) => {
        const { result } = renderHook(() =>
            useCommandExecution({
                query,
                results: [],
                selectedIndex: 0,
                argumentMode: { alias: 's', command: search },
                setQuery: jest.fn(),
                setSearchMode: jest.fn(),
                resetState: jest.fn(),
                executeCommand,
            }),
        );
        return result;
    };

    it('does not run until every placeholder has a value', async () => {
        const executeCommand = jest.fn().mockResolvedValue(true);
        const result = setupArguments('s ', executeCommand);
        await act(() => result.current.runSelectedCommand());
        expect(executeCommand).not.toHaveBeenCalled();
    });

    it('runs with the typed arguments once they are all there', async () => {
        const executeCommand = jest.fn().mockResolvedValue(true);
        const result = setupArguments('s react', executeCommand);
        await act(() => result.current.runSelectedCommand());
        expect(executeCommand).toHaveBeenCalledWith(search, ['react']);
    });
});

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
