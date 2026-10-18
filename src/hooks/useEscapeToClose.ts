import { useEffect } from 'react';

/**
 * Runs `onEscape` when Esc is pressed on a full screen (the config screen, the command form).
 * An Esc that something closer already handled (`preventDefault`), or that belongs to an open
 * dialog, is left alone, so it never closes the screen behind it as well.
 */
export const useEscapeToClose = (onEscape: () => void, enabled = true) => {
    useEffect(() => {
        if (!enabled) return;
        const handleKeyDown = (e: KeyboardEvent) => {
            if (e.key !== 'Escape' || e.defaultPrevented || e.isComposing) return;
            if (document.querySelector('dialog[open]')) return;
            // Without this the same Esc would also close a dialog that onEscape opens
            e.preventDefault();
            onEscape();
        };
        window.addEventListener('keydown', handleKeyDown);
        return () => window.removeEventListener('keydown', handleKeyDown);
    }, [onEscape, enabled]);
};
