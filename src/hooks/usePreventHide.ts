import { useEffect } from 'react';
import { TauriAPI } from '../api/tauri';

/**
 * ダイアログなどの表示中にウィンドウが自動的に隠れるのを防ぐためのフック
 * @param shouldPrevent trueの場合、ウィンドウの自動非表示を無効化する
 */
export const usePreventHide = (shouldPrevent: boolean) => {
    useEffect(() => {
        if (shouldPrevent) {
            TauriAPI.setPreventHide(true);
        } else {
            TauriAPI.setPreventHide(false);
        }

        return () => {
            // shouldPreventがtrueだった場合のクリーンアップ、
            // またはコンポーネントのアンマウント時にfalseに戻す
            if (shouldPrevent) {
                TauriAPI.setPreventHide(false);
            }
        };
    }, [shouldPrevent]);
};
