import { useEffect } from 'react';
import { TauriAPI } from '../api/tauri';

export const useWindowEvents = (isDialogOpen: boolean, resetState: () => void, inputRef: React.RefObject<HTMLInputElement | null>) => {
    useEffect(() => {
        inputRef.current?.focus();

        const handleDocumentClick = (e: MouseEvent) => {
            if (isDialogOpen) return;

            const target = e.target as HTMLElement;
            if (target.closest('.dropdown, .dropdown-content')) {
                return;
            }

            const searchWindow = document.querySelector('.h-full');
            if (searchWindow) {
                const rect = searchWindow.getBoundingClientRect();
                const clickX = e.clientX;
                const clickY = e.clientY;

                const isOutside = clickX < rect.left ||
                    clickX > rect.right ||
                    clickY < rect.top ||
                    clickY > rect.bottom;

                if (isOutside) {
                    TauriAPI.hideWindow();
                }
            }
        };

        const handleGlobalKeyDown = (e: KeyboardEvent) => {
            if (isDialogOpen) return;

            if (e.altKey && e.code === 'Space') {
                e.preventDefault();
                TauriAPI.hideWindow();
            }
        };

        const setupTauriListeners = async () => {
            try {
                const { listen } = await import('@tauri-apps/api/event');
                await listen('window-shown', resetState);
            } catch (error) {
                console.warn('Failed to setup Tauri event listeners:', error);
            }
        };

        setupTauriListeners();
        document.addEventListener('click', handleDocumentClick, true);
        document.addEventListener('keydown', handleGlobalKeyDown);

        return () => {
            document.removeEventListener('click', handleDocumentClick, true);
            document.removeEventListener('keydown', handleGlobalKeyDown);
        };
    }, [resetState, isDialogOpen, inputRef]);
};
