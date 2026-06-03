import { renderHook, act } from '@testing-library/react';
import { useSearchState } from '../useSearchState';
import { Command } from '../../types';

const makeCmd = (name: string, id = name, alias?: string): Command => ({
    id,
    name,
    category: 'url',
    source: 'user',
    command: 'https://example.com',
    description: '',
    alias,
});

const THRESHOLD = 0.4;
const MAX = 10;

describe('useSearchState', () => {
    describe('initial state', () => {
        it('starts in idle mode with empty query', () => {
            const { result } = renderHook(() => useSearchState([], THRESHOLD, MAX));
            expect(result.current.query).toBe('');
            expect(result.current.searchMode.mode).toBe('idle');
        });
    });

    describe('search mode', () => {
        const commands = [makeCmd('GitHub'), makeCmd('Google'), makeCmd('Gmail')];

        it('transitions to search mode on query input', () => {
            const { result } = renderHook(() => useSearchState(commands, THRESHOLD, MAX));
            act(() => {
                result.current.handleQueryChange('git');
            });
            expect(result.current.searchMode.mode).toBe('search');
            expect(result.current.results.length).toBeGreaterThan(0);
        });

        it('returns to idle when query is cleared', () => {
            const { result } = renderHook(() => useSearchState(commands, THRESHOLD, MAX));
            act(() => {
                result.current.handleQueryChange('git');
            });
            act(() => {
                result.current.handleQueryChange('');
            });
            expect(result.current.searchMode.mode).toBe('idle');
        });

        it('selectedIndex starts at 0', () => {
            const { result } = renderHook(() => useSearchState(commands, THRESHOLD, MAX));
            act(() => {
                result.current.handleQueryChange('g');
            });
            expect(result.current.selectedIndex).toBe(0);
        });
    });

    describe('argument mode transition', () => {
        const aliasCmd = makeCmd('Open URL', 'open-url', 'open');
        const commands = [aliasCmd, makeCmd('GitHub')];

        it('transitions to argument mode when alias + space is typed', () => {
            const { result } = renderHook(() => useSearchState(commands, THRESHOLD, MAX));
            act(() => {
                result.current.handleQueryChange('open ');
            });
            expect(result.current.searchMode.mode).toBe('argument');
            expect(result.current.argumentMode?.alias).toBe('open');
            expect(result.current.argumentMode?.command.id).toBe('open-url');
        });

        it('stays in search mode while typing the alias (no trailing space)', () => {
            const { result } = renderHook(() => useSearchState(commands, THRESHOLD, MAX));
            act(() => {
                result.current.handleQueryChange('open');
            });
            expect(result.current.searchMode.mode).toBe('search');
        });

        it('stays in argument mode when typing args after the alias prefix', () => {
            const { result } = renderHook(() => useSearchState(commands, THRESHOLD, MAX));
            act(() => {
                result.current.handleQueryChange('open ');
            });
            act(() => {
                result.current.handleQueryChange('open https://example.com');
            });
            expect(result.current.searchMode.mode).toBe('argument');
        });

        it('argumentMode is null outside argument mode', () => {
            const { result } = renderHook(() => useSearchState(commands, THRESHOLD, MAX));
            expect(result.current.argumentMode).toBeNull();
            act(() => {
                result.current.handleQueryChange('git');
            });
            expect(result.current.argumentMode).toBeNull();
        });
    });

    describe('resetState', () => {
        const commands = [makeCmd('GitHub')];

        it('resets to idle with empty query', () => {
            const { result } = renderHook(() => useSearchState(commands, THRESHOLD, MAX));
            act(() => {
                result.current.handleQueryChange('git');
            });
            act(() => {
                result.current.resetState();
            });
            expect(result.current.query).toBe('');
            expect(result.current.searchMode.mode).toBe('idle');
        });

        it('resets from argument mode to idle', () => {
            const aliasCmd = makeCmd('Open URL', 'open-url', 'open');
            const { result } = renderHook(() => useSearchState([aliasCmd], THRESHOLD, MAX));
            act(() => {
                result.current.handleQueryChange('open ');
            });
            expect(result.current.searchMode.mode).toBe('argument');
            act(() => {
                result.current.resetState();
            });
            expect(result.current.searchMode.mode).toBe('idle');
            expect(result.current.query).toBe('');
        });
    });

    describe('moveSelection', () => {
        const commands = [makeCmd('Alpha'), makeCmd('Beta'), makeCmd('Gamma')];

        it('moves selection down', () => {
            const { result } = renderHook(() => useSearchState(commands, THRESHOLD, MAX));
            act(() => {
                result.current.handleQueryChange('a');
            });
            const initialIndex = result.current.selectedIndex;
            act(() => {
                result.current.moveSelection('down');
            });
            expect(result.current.selectedIndex).toBe(initialIndex + 1);
        });

        it('wraps selection down from last to first', () => {
            const { result } = renderHook(() => useSearchState(commands, THRESHOLD, MAX));
            act(() => {
                result.current.handleQueryChange('a');
            });
            const count = result.current.results.length;
            expect(count).toBeGreaterThan(1);
            // move to last
            for (let i = 0; i < count - 1; i++) {
                act(() => result.current.moveSelection('down'));
            }
            expect(result.current.selectedIndex).toBe(count - 1);
            act(() => {
                result.current.moveSelection('down');
            });
            expect(result.current.selectedIndex).toBe(0);
        });

        it('wraps selection up from first to last', () => {
            const { result } = renderHook(() => useSearchState(commands, THRESHOLD, MAX));
            act(() => {
                result.current.handleQueryChange('a');
            });
            const count = result.current.results.length;
            expect(count).toBeGreaterThan(1);
            act(() => {
                result.current.moveSelection('up');
            });
            expect(result.current.selectedIndex).toBe(count - 1);
        });

        it('does not change index when not in search mode', () => {
            const { result } = renderHook(() => useSearchState(commands, THRESHOLD, MAX));
            act(() => {
                result.current.moveSelection('down');
            });
            expect(result.current.searchMode.mode).toBe('idle');
            expect(result.current.selectedIndex).toBe(0);
        });
    });

    describe('commands prop change', () => {
        it('recomputes search results when commands change', () => {
            const initial = [makeCmd('Alpha')];
            const { result, rerender } = renderHook(
                ({ cmds }: { cmds: Command[] }) => useSearchState(cmds, THRESHOLD, MAX),
                { initialProps: { cmds: initial } },
            );
            act(() => {
                result.current.handleQueryChange('beta');
            });
            expect(result.current.results.length).toBe(0);

            rerender({ cmds: [makeCmd('Alpha'), makeCmd('Beta')] });

            expect(result.current.results.some((r) => r.command.name === 'Beta')).toBe(true);
        });
    });
});
