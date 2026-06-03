import { useEffect } from 'react';
import { TauriAPI } from '../tauri';

export const useWindowEvents = (
    isDialogOpen: boolean,
    resetState: () => void,
    inputRef: React.RefObject<HTMLInputElement | null>,
) => {
    useEffect(() => {
        inputRef.current?.focus();
    }, [inputRef]);

    useEffect(() => {
        const handleDocumentClick = (e: MouseEvent) => {
            if (isDialogOpen) return;

            const target = e.target as HTMLElement;
            if (target.closest('.dropdown, .dropdown-content')) return;

            const searchWindow = document.querySelector('.h-full');
            if (searchWindow) {
                const rect = searchWindow.getBoundingClientRect();
                const isOutside =
                    e.clientX < rect.left ||
                    e.clientX > rect.right ||
                    e.clientY < rect.top ||
                    e.clientY > rect.bottom;

                if (isOutside) {
                    TauriAPI.hideWindow();
                }
            }
        };

        document.addEventListener('click', handleDocumentClick, true);
        return () => document.removeEventListener('click', handleDocumentClick, true);
    }, [isDialogOpen]);

    useEffect(() => {
        const handleKeyDown = (e: KeyboardEvent) => {
            if (isDialogOpen) return;
            if (e.altKey && e.code === 'Space') {
                e.preventDefault();
                TauriAPI.hideWindow();
            }
        };

        document.addEventListener('keydown', handleKeyDown);
        return () => document.removeEventListener('keydown', handleKeyDown);
    }, [isDialogOpen]);

    useEffect(() => {
        let unlisten: (() => void) | undefined;

        import('@tauri-apps/api/event')
            .then(({ listen }) => listen('window-shown', resetState))
            .then((fn) => {
                unlisten = fn;
            })
            .catch((error) => console.warn('Failed to setup window-shown listener:', error));

        return () => {
            unlisten?.();
        };
    }, [resetState]);
};
