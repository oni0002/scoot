import { useEffect } from 'react';
import { listen } from '@tauri-apps/api/event';
import { TauriAPI } from '../tauri';
import { getErrorMessage } from '../error';
import { Notice } from '../types';
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
        // What the user was last told, per source, so opening the window does not repeat it
        const told = new Map<string, string>();
        const tell = (notice: Notice) => {
            told.set(notice.source, notice.message);
            const message = getErrorMessage(notice.message);
            if (notice.level === 'error') showError(message, NOTIFICATION_DURATION.VERY_LONG);
            else showWarning(message, NOTIFICATION_DURATION.LONG);
        };
        // A notice raised while the window was hidden (startup, a timer, a file change) would have
        // expired unseen: it waits in the backend until the window is shown
        const tellNew = (notices: Notice[]) => {
            const sources = new Set(notices.map((n) => n.source));
            for (const source of [...told.keys()]) if (!sources.has(source)) told.delete(source);
            notices.filter((n) => told.get(n.source) !== n.message).forEach(tell);
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
                    // Sent only while the window is showing. settings.json or commands.json could not
                    // be read (the last good state stays in use), or the Hotkey could not be taken.
                    notice: (event: { payload: Notice }) => tell(event.payload),
                    'window-shown': () => {
                        TauriAPI.getNotices().then(tellNew, (err) =>
                            console.warn('Failed to read the notices:', err),
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
