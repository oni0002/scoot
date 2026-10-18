import { useEffect } from 'react';
import { listen } from '@tauri-apps/api/event';
import { NOTIFICATION_DURATION } from '../constants';

interface UseAppEventsProps {
    handleAddCommand: () => void;
    handleOpenConfig: () => void;
    loadCommands: () => void;
    loadConfig: () => void;
    showWarning: (message: string) => void;
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
                        showWarning(event.payload);
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
