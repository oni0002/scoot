import React, { useCallback, useEffect, useState } from 'react';
import { LuCheck, LuTriangleAlert, LuCircleX, LuInfo } from 'react-icons/lu';

export interface Notification {
    id: string;
    type: 'success' | 'error' | 'warning' | 'info';
    message: string;
    duration?: number;
    /** A single follow-up the user can take from the toast, e.g. Undo. */
    action?: { label: string; onClick: () => void };
}

interface NotificationToastProps {
    notifications: Notification[];
    onRemove: (id: string) => void;
}

export const NotificationToast: React.FC<NotificationToastProps> = ({
    notifications,
    onRemove,
}) => {
    return (
        <div className="toast toast-bottom toast-end z-50" aria-live="polite">
            {notifications.map((notification) => (
                <NotificationItem
                    key={notification.id}
                    notification={notification}
                    onRemove={onRemove}
                />
            ))}
        </div>
    );
};

interface NotificationItemProps {
    notification: Notification;
    onRemove: (id: string) => void;
}

const NotificationItem: React.FC<NotificationItemProps> = ({ notification, onRemove }) => {
    const [isVisible, setIsVisible] = useState(false);

    const handleRemove = useCallback(() => {
        setIsVisible(false);
        setTimeout(() => {
            onRemove(notification.id);
        }, 300); // Wait for animation
    }, [onRemove, notification.id]);

    useEffect(() => {
        // Animate in
        const timer = setTimeout(() => setIsVisible(true), 10);

        // Auto-remove after duration
        const duration = notification.duration || 5000;
        const removeTimer = setTimeout(() => {
            handleRemove();
        }, duration);

        return () => {
            clearTimeout(timer);
            clearTimeout(removeTimer);
        };
    }, [notification.duration, handleRemove]);

    const getAlertClass = () => {
        switch (notification.type) {
            case 'success':
                return 'alert-success';
            case 'error':
                return 'alert-error';
            case 'warning':
                return 'alert-warning';
            case 'info':
                return 'alert-info';
            default:
                return 'alert-info';
        }
    };

    const getIcon = () => {
        switch (notification.type) {
            case 'success':
                return <LuCheck />;
            case 'error':
                return <LuCircleX />;
            case 'warning':
                return <LuTriangleAlert />;
            case 'info':
                return <LuInfo />;
            default:
                return <LuInfo />;
        }
    };

    return (
        <div
            role={notification.type === 'error' ? 'alert' : 'status'}
            className={`alert ${getAlertClass()} rounded-lg transition-all duration-300 max-w-sm py-2 px-3 gap-2 shadow-lg flex items-center text-left ${
                isVisible ? 'opacity-100 translate-x-0' : 'opacity-0 translate-x-full'
            }`}
        >
            {getIcon()}
            <div className="min-w-0 flex-1">
                <div className="text-sm leading-snug whitespace-normal break-words">
                    {notification.message}
                </div>
            </div>
            {notification.action && (
                <button
                    type="button"
                    className="btn btn-ghost btn-xs text-inherit underline"
                    onClick={() => {
                        notification.action?.onClick();
                        handleRemove();
                    }}
                >
                    {notification.action.label}
                </button>
            )}
        </div>
    );
};
