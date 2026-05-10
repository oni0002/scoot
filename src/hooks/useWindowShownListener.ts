import { useEffect } from 'react';

export const useWindowShownListener = (onWindowShown: () => void) => {
  useEffect(() => {
    let unlisten: (() => void) | undefined;

    import('@tauri-apps/api/event')
      .then(({ listen }) => listen('window-shown', onWindowShown))
      .then((fn) => { unlisten = fn; })
      .catch((error) => console.warn('Failed to setup window-shown listener:', error));

    return () => { unlisten?.(); };
  }, [onWindowShown]);
};
