import { useEffect } from 'react';

/**
 * Puts focus back on the search field when something that held it (a row menu, say) has gone and
 * left it on the page body. Never while the window itself is unfocused: calling focus() then would
 * pull the OS focus back to the launcher and stop it from hiding when the user clicks away.
 */
export const useRestoreFocus = (
    inputRef: React.RefObject<HTMLInputElement | null>,
    shouldRestore: boolean,
) => {
    useEffect(() => {
        if (shouldRestore && document.hasFocus() && document.activeElement === document.body) {
            inputRef.current?.focus();
        }
    }, [shouldRestore, inputRef]);
};
