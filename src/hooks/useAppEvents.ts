import { useEffect } from 'react';
import { listen } from '@tauri-apps/api/event';
import { NOTIFICATION_DURATION } from '../constants';

interface UseAppEventsProps {
    handleAddCommand: () => void;
    loadCommands: () => void;
    loadConfig: () => void;
    showSuccess: (message: string) => void;
    showWarning: (message: string) => void;
    showInfo: (message: string) => void;
    showError: (message: string, duration?: number) => void;
    setError: (error: string | null) => void;
}

export const useAppEvents = ({
    handleAddCommand,
    loadCommands,
    loadConfig,
    showSuccess,
    showWarning,
    showInfo,
    showError,
    setError,
}: UseAppEventsProps) => {
    useEffect(() => {
        // Initial load
        loadCommands();
        loadConfig();

        let unlistenFunctions: (() => void)[] = [];
        let isMounted = true;

        const setupEventListeners = async () => {
            try {
                const eventHandlers = {
                    "open-add-command-dialog": handleAddCommand,
                    "config-reloaded": () => {
                        console.log("Event: config-reloaded received");
                        showInfo("Commands reloaded from file.");
                        loadCommands();
                        loadConfig();
                    },
                    "config-file-changed": () => {
                        console.log("Event: config-file-changed received");
                        loadCommands();
                        loadConfig();
                    },
                    "shortcut-registered": (event: any) => {
                        showSuccess(`Global shortcut: ${event.payload}`);
                    },
                    "shortcut-warning": () => {
                        showWarning("Use system tray to open.");
                    }
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
            } catch (err) {
                if (isMounted) {
                    console.error("Failed to setup event listeners", err);
                    setError("Failed to setup event listeners");
                    showError("Failed to setup event listeners", NOTIFICATION_DURATION.VERY_LONG);
                }
            };
        };

        setupEventListeners();

        return () => {
            isMounted = false;
            unlistenFunctions.forEach(fn => fn());
        };
    }, []); // Run only once on mount
};
