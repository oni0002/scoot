import { useCallback } from 'react';
import { KEYBOARD_SHORTCUTS } from '../constants';
import { TauriAPI } from '../tauri';
import { SearchMode } from './useSearchState';

interface UseKeyboardNavigationProps {
    moveSelection: (direction: 'up' | 'down') => void;
    executeCommand: () => void;
    resetState: () => void;
    query: string;
    setQuery: (query: string) => void;
    searchMode: SearchMode;
    setSearchMode: (mode: SearchMode) => void;
}

export const useKeyboardNavigation = ({
    moveSelection,
    executeCommand,
    resetState,
    query,
    setQuery,
    searchMode,
    setSearchMode,
}: UseKeyboardNavigationProps) => {
    const handleKeyDown = useCallback(
        (e: React.KeyboardEvent) => {
            if (e.ctrlKey) {
                if (KEYBOARD_SHORTCUTS.MOVE_DOWN_ALT.includes(e.key)) {
                    e.preventDefault();
                    moveSelection('down');
                    return;
                }
                if (KEYBOARD_SHORTCUTS.MOVE_UP_ALT.includes(e.key)) {
                    e.preventDefault();
                    moveSelection('up');
                    return;
                }
            }

            if (KEYBOARD_SHORTCUTS.MOVE_DOWN.includes(e.key)) {
                e.preventDefault();
                moveSelection('down');
            } else if (KEYBOARD_SHORTCUTS.MOVE_UP.includes(e.key)) {
                e.preventDefault();
                moveSelection('up');
            } else if (KEYBOARD_SHORTCUTS.NAVIGATE_TAB.includes(e.key)) {
                e.preventDefault();
                moveSelection(e.shiftKey ? 'up' : 'down');
            } else if (KEYBOARD_SHORTCUTS.EXECUTE.includes(e.key)) {
                e.preventDefault();
                executeCommand();
            } else if (KEYBOARD_SHORTCUTS.CANCEL.includes(e.key)) {
                e.preventDefault();
                if (searchMode.mode === 'argument') {
                    resetState();
                } else if (query) {
                    setQuery('');
                    setSearchMode({ mode: 'idle' });
                } else {
                    TauriAPI.hideWindow();
                }
            }
        },
        [moveSelection, executeCommand, searchMode, query, resetState, setQuery, setSearchMode],
    );

    return { handleKeyDown };
};
