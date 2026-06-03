import { useCallback } from 'react';
import React from 'react';
import { KEYBOARD_SHORTCUTS } from '../constants';
import { TauriAPI } from '../tauri';
import { SearchMode } from './useSearchState';

interface UseKeyboardNavigationProps {
    moveSelection: (direction: 'up' | 'down') => void;
    executeCommand: () => void;
    copySelectedCommand?: () => void;
    resetState: () => void;
    query: string;
    setQuery: (query: string) => void;
    searchMode: SearchMode;
    setSearchMode: (mode: SearchMode) => void;
    inputRef: React.RefObject<HTMLInputElement | null>;
}

export const useKeyboardNavigation = ({
    moveSelection,
    executeCommand,
    copySelectedCommand,
    resetState,
    query,
    setQuery,
    searchMode,
    setSearchMode,
    inputRef,
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
                if (KEYBOARD_SHORTCUTS.COPY.includes(e.key)) {
                    const input = inputRef.current;
                    const hasSelection = input && input.selectionStart !== input.selectionEnd;
                    if (!hasSelection) {
                        e.preventDefault();
                        copySelectedCommand?.();
                    }
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
        [moveSelection, executeCommand, copySelectedCommand, searchMode, query, resetState, setQuery, setSearchMode, inputRef],
    );

    return { handleKeyDown };
};
