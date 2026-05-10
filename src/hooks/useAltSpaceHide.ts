import { useEffect } from 'react';
import { TauriAPI } from '../api/tauri';

export const useAltSpaceHide = (isDialogOpen: boolean) => {
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
};
