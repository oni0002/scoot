import { useState, useCallback } from 'react';
import { Notification } from '../components/NotificationToast';
import { getErrorMessage } from '../error';
import { NOTIFICATION_DURATION } from '../constants';

let notificationId = 0;

export const useNotifications = () => {
    const [notifications, setNotifications] = useState<Notification[]>([]);

    const addNotification = useCallback(
        (
            type: Notification['type'],
            message: string,
            duration?: number,
            action?: Notification['action'],
        ) => {
            const id = `notification-${++notificationId}`;
            const notification: Notification = {
                id,
                type,
                message,
                duration,
                action,
            };

            setNotifications((prev) => [...prev, notification]);
            return id;
        },
        [],
    );

    const removeNotification = useCallback((id: string) => {
        setNotifications((prev) => prev.filter((n) => n.id !== id));
    }, []);

    // Convenience methods
    const showSuccess = useCallback(
        (message: string, duration?: number, action?: Notification['action']) => {
            return addNotification('success', message, duration, action);
        },
        [addNotification],
    );

    const showError = useCallback(
        (message: string, duration?: number, action?: Notification['action']) => {
            return addNotification(
                'error',
                message,
                duration || NOTIFICATION_DURATION.LONG,
                action,
            ); // Errors stay longer
        },
        [addNotification],
    );

    const showWarning = useCallback(
        (message: string, duration?: number) => {
            return addNotification('warning', message, duration);
        },
        [addNotification],
    );

    /** A toast for work still running. It stays until the returned function is called. */
    const showProgress = useCallback(
        (message: string) => {
            const id = addNotification('progress', message, NOTIFICATION_DURATION.PROGRESS_MAX);
            return () => removeNotification(id);
        },
        [addNotification, removeNotification],
    );

    /** Logs a failure and tells the user: "Could not <what>: <the reason>". */
    const reportError = useCallback(
        (what: string, err: unknown, duration?: number, action?: Notification['action']) => {
            console.error(`Failed to ${what}:`, err);
            return showError(`Could not ${what}: ${getErrorMessage(err)}`, duration, action);
        },
        [showError],
    );

    return {
        notifications,
        removeNotification,
        showSuccess,
        showError,
        showWarning,
        showProgress,
        reportError,
    };
};
