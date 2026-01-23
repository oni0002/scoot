import { useCallback } from 'react';
import { KEYBOARD_SHORTCUTS } from '../constants';
import { TauriAPI } from '../api/tauri';

interface UseKeyboardNavigationProps {
    moveSelection: (direction: 'up' | 'down') => void;
    executeCommand: () => void;
    resetState: () => void;
    query: string;
    setQuery: (query: string) => void;
    setResults: (results: any[]) => void;
    setSelectedIndex: (index: number) => void;
    promptMode: any;
}

export const useKeyboardNavigation = ({
    moveSelection,
    executeCommand,
    resetState,
    query,
    setQuery,
    setResults,
    setSelectedIndex,
    promptMode
}: UseKeyboardNavigationProps) => {

    const handleKeyDown = useCallback((e: React.KeyboardEvent) => {
        // Ctrl+N/P for navigation
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
            if (promptMode) {
                resetState();
            } else if (query) {
                setQuery('');
                setResults([]);
                setSelectedIndex(0);
            } else {
                TauriAPI.hideWindow();
            }
        }
    }, [moveSelection, executeCommand, promptMode, query, resetState, setQuery, setResults, setSelectedIndex]);

    return { handleKeyDown };
};
