import { useState, useCallback } from 'react';
import { Notification } from '../components/NotificationToast';

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
            return addNotification('error', message, duration || 8000, action); // Errors stay longer
        },
        [addNotification],
    );

    const showWarning = useCallback(
        (message: string, duration?: number) => {
            return addNotification('warning', message, duration);
        },
        [addNotification],
    );

    return {
        notifications,
        removeNotification,
        showSuccess,
        showError,
        showWarning,
    };
};
