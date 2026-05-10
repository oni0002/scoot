import { useEffect } from 'react';
import { TauriAPI } from '../api/tauri';

export const useClickOutsideHide = (isDialogOpen: boolean) => {
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
};
