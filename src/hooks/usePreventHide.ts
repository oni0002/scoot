import { useEffect } from 'react';
import { TauriAPI } from '../api/tauri';

/**
 * hook to prevent window from auto-hiding while dialogs are shown
 * @param shouldPrevent if true, disable auto-hide
 */
export const usePreventHide = (shouldPrevent: boolean) => {
    useEffect(() => {
        if (shouldPrevent) {
            TauriAPI.setPreventHide(true);
        } else {
            TauriAPI.setPreventHide(false);
        }

        return () => {
            // cleanup for when shouldPrevent was true, or when component unmounts
            if (shouldPrevent) {
                TauriAPI.setPreventHide(false);
            }
        };
    }, [shouldPrevent]);
};
