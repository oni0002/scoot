import { useEffect } from 'react';
import { TauriAPI } from '../api/tauri';

export const usePreventHide = (shouldPrevent: boolean) => {
    useEffect(() => {
        if (!shouldPrevent) return;
        TauriAPI.enterModal();
        return () => { TauriAPI.leaveModal(); };
    }, [shouldPrevent]);
};
