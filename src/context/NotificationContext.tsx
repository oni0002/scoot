import React, { createContext, useContext, useMemo, ReactNode } from 'react';
import { useNotifications } from '../hooks/useNotifications';
import { Notification, NotificationToast } from '../components/NotificationToast';
import { useSettingsTheme } from './SettingsContext';

interface NotificationContextType {
    showSuccess: (message: string, duration?: number, action?: Notification['action']) => void;
    showError: (message: string, duration?: number, action?: Notification['action']) => void;
    showWarning: (message: string, duration?: number) => void;
    /** Shows a spinner toast until the returned function is called. */
    showProgress: (message: string) => () => void;
    /** Logs a failure and shows "Could not <what>: <the reason>". */
    reportError: (
        what: string,
        err: unknown,
        duration?: number,
        action?: Notification['action'],
    ) => void;
}

const NotificationContext = createContext<NotificationContextType | undefined>(undefined);

export const NotificationProvider: React.FC<{ children: ReactNode }> = ({ children }) => {
    const theme = useSettingsTheme();
    const {
        notifications,
        removeNotification,
        showSuccess,
        showError,
        showWarning,
        showProgress,
        reportError,
    } = useNotifications();

    const value = useMemo(
        () => ({ showSuccess, showError, showWarning, showProgress, reportError }),
        [showSuccess, showError, showWarning, showProgress, reportError],
    );

    return (
        <NotificationContext.Provider value={value}>
            {children}
            <NotificationToast
                notifications={notifications}
                onRemove={removeNotification}
                theme={theme}
            />
        </NotificationContext.Provider>
    );
};

export const useNotificationContext = () => {
    const context = useContext(NotificationContext);
    if (context === undefined) {
        throw new Error('useNotificationContext must be used within a NotificationProvider');
    }
    return context;
};
