import { useEffect } from 'react';
import { listen } from '@tauri-apps/api/event';
import { TauriAPI } from '../tauri';
import { NOTIFICATION_DURATION } from '../constants';

interface UseAppEventsProps {
    handleAddCommand: () => void;
    handleOpenConfig: () => void;
    loadCommands: () => void;
    loadConfig: () => void;
    showWarning: (message: string, duration?: number) => void;
    showError: (message: string, duration?: number) => void;
    setError: (error: string | null) => void;
}

export const useAppEvents = ({
    handleAddCommand,
    handleOpenConfig,
    loadCommands,
    loadConfig,
    showWarning,
    showError,
    setError,
}: UseAppEventsProps) => {
    useEffect(() => {
        const unlistenFunctions: (() => void)[] = [];
        let isMounted = true;
        // The last Hotkey warning the user was shown, so opening the window does not repeat it
        let shownWarning: string | null = null;
        const warnOnce = (warning: string | null) => {
            if (!warning || warning === shownWarning) return;
            shownWarning = warning;
            showWarning(warning, NOTIFICATION_DURATION.LONG);
        };

        const setupEventListeners = async () => {
            try {
                const eventHandlers = {
                    'open-add-command-dialog': handleAddCommand,
                    'open-settings-dialog': handleOpenConfig,
                    // The backend turns config file changes into a reload, so this is the only refetch
                    'config-reloaded': () => {
                        loadCommands();
                        loadConfig();
                    },
                    // config.json or commands.json could not be read; the last good state stays in use
                    'config-load-error': (event: { payload: string }) => {
                        showError(event.payload, NOTIFICATION_DURATION.VERY_LONG);
                    },
                    // The Hotkey could not be taken (another app holds it); the message says which
                    'shortcut-warning': (event: { payload: string }) => {
                        shownWarning = event.payload;
                        showWarning(event.payload, NOTIFICATION_DURATION.LONG);
                    },
                    // The Hotkey is registered at startup, while the window is hidden and nobody
                    // would see a notice. Whoever opens the window (tray, second launch) sees it.
                    'window-shown': () => {
                        TauriAPI.getShortcutWarning().then(warnOnce, (err) =>
                            console.warn('Failed to read the hotkey status:', err),
                        );
                    },
                };

                for (const [event, handler] of Object.entries(eventHandlers)) {
                    if (!isMounted) return;

                    const unlisten = await listen(event, handler);

                    if (isMounted) {
                        unlistenFunctions.push(unlisten);
                    } else {
                        unlisten();
                    }
                }
                // Load only once subscribed, so a reload finishing during startup cannot be missed
                if (isMounted) {
                    loadCommands();
                    loadConfig();
                }
            } catch (err) {
                if (isMounted) {
                    console.error('Failed to setup event listeners', err);
                    setError('Failed to setup event listeners');
                    showError('Failed to setup event listeners', NOTIFICATION_DURATION.VERY_LONG);
                }
            }
        };

        setupEventListeners();

        return () => {
            isMounted = false;
            unlistenFunctions.forEach((fn) => fn());
        };
        // eslint-disable-next-line react-hooks/exhaustive-deps
    }, []); // all passed handlers are useCallback-stabilized with empty dep arrays
};
