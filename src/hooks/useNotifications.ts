import { useState, useCallback } from 'react';
import { Notification } from '../components/NotificationToast';

let notificationId = 0;

export const useNotifications = () => {
    const [notifications, setNotifications] = useState<Notification[]>([]);

    const addNotification = useCallback(
        (type: Notification['type'], message: string, duration?: number) => {
            const id = `notification-${++notificationId}`;
            const notification: Notification = {
                id,
                type,
                message,
                duration,
            };

            setNotifications((prev) => [...prev, notification]);
            return id;
        },
        [],
    );

    const removeNotification = useCallback((id: string) => {
        setNotifications((prev) => prev.filter((n) => n.id !== id));
    }, []);

    const clearAll = useCallback(() => {
        setNotifications([]);
    }, []);

    // Convenience methods
    const showSuccess = useCallback(
        (message: string, duration?: number) => {
            return addNotification('success', message, duration);
        },
        [addNotification],
    );

    const showError = useCallback(
        (message: string, duration?: number) => {
            return addNotification('error', message, duration || 8000); // Errors stay longer
        },
        [addNotification],
    );

    const showWarning = useCallback(
        (message: string, duration?: number) => {
            return addNotification('warning', message, duration);
        },
        [addNotification],
    );

    const showInfo = useCallback(
        (message: string, duration?: number) => {
            return addNotification('info', message, duration);
        },
        [addNotification],
    );

    return {
        notifications,
        addNotification,
        removeNotification,
        clearAll,
        showSuccess,
        showError,
        showWarning,
        showInfo,
    };
};
