import { useCallback } from 'react';
import React from 'react';
import { KEYBOARD_SHORTCUTS } from '../constants';
import { TauriAPI } from '../tauri';
import { SearchMode } from './useSearchState';
import { ROW_ACTION_BY_KEY, RowAction } from '../rowActions';

interface UseKeyboardNavigationProps {
    moveSelection: (direction: 'up' | 'down') => void;
    executeCommand: () => void;
    copySelectedCommand?: () => void;
    onRowAction?: (action: Exclude<RowAction, 'copy'>) => void;
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
    onRowAction,
    resetState,
    query,
    setQuery,
    searchMode,
    setSearchMode,
    inputRef,
}: UseKeyboardNavigationProps) => {
    const handleKeyDown = useCallback(
        (e: React.KeyboardEvent) => {
            // Enter, arrows and Esc while composing text (IME) belong to the composition
            if (e.nativeEvent.isComposing) return;

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
                const rowAction = e.altKey ? undefined : ROW_ACTION_BY_KEY[e.key.toLowerCase()];
                if (rowAction) {
                    e.preventDefault();
                    onRowAction?.(rowAction);
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
        [
            moveSelection,
            executeCommand,
            copySelectedCommand,
            onRowAction,
            searchMode,
            query,
            resetState,
            setQuery,
            setSearchMode,
            inputRef,
        ],
    );

    return { handleKeyDown };
};
