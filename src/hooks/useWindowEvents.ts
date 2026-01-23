import { useEffect } from 'react';
import { TauriAPI } from '../api/tauri';

export const useWindowEvents = (isDialogOpen: boolean, resetState: () => void, inputRef: React.RefObject<HTMLInputElement | null>) => {
    useEffect(() => {
        inputRef.current?.focus();

        const handleDocumentClick = (e: MouseEvent) => {
            // ダイアログが開いている場合はウィンドウを閉じない
            if (isDialogOpen) return;

            // メニューのクリックは無視
            const target = e.target as HTMLElement;
            if (target.closest('.dropdown, .dropdown-content')) {
                return;
            }

            // SearchWindow要素の境界を正確に判定
            const searchWindow = document.querySelector('.h-full');
            if (searchWindow) {
                const rect = searchWindow.getBoundingClientRect();
                const clickX = e.clientX;
                const clickY = e.clientY;

                // クリック位置がSearchWindow要素の境界外かどうかを確認
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
            // ダイアログが開いている場合はグローバルキーを無効化
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
