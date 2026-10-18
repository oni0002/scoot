import React, { useCallback, useEffect, useState } from 'react';
import { daisyTheme } from '../theme';
import { NOTIFICATION_DURATION } from '../constants';
import { LuX, LuCheck, LuTriangleAlert, LuCircleX } from 'react-icons/lu';

export interface Notification {
    id: string;
    type: 'success' | 'error' | 'warning';
    message: string;
    duration?: number;
    /** A single follow-up the user can take from the toast, e.g. Undo. */
    action?: { label: string; onClick: () => void };
}

// A quiet surface with a coloured icon: the filled daisyUI alerts put white text on strong
// colours, which made the Undo action hard to read and shouted over the launcher's palette.
const TYPE_STYLE: Record<Notification['type'], { icon: React.ReactNode; className: string }> = {
    success: { icon: <LuCheck />, className: 'text-success' },
    error: { icon: <LuCircleX />, className: 'text-error' },
    warning: { icon: <LuTriangleAlert />, className: 'text-warning' },
};

interface NotificationToastProps {
    notifications: Notification[];
    onRemove: (id: string) => void;
    /** Settings theme value; the toast lives outside the themed app root, so it needs its own. */
    theme?: string;
}

export const NotificationToast: React.FC<NotificationToastProps> = ({
    notifications,
    onRemove,
    theme,
}) => {
    return (
        <div
            className="toast toast-bottom toast-end z-50"
            aria-live="polite"
            data-theme={theme ? daisyTheme(theme) : undefined}
        >
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
        const duration = notification.duration || NOTIFICATION_DURATION.DEFAULT;
        const removeTimer = setTimeout(() => {
            handleRemove();
        }, duration);

        return () => {
            clearTimeout(timer);
            clearTimeout(removeTimer);
        };
    }, [notification.duration, handleRemove]);

    return (
        <div
            role={notification.type === 'error' ? 'alert' : 'status'}
            className={`alert bg-base-100 text-base-content border border-base-content/30 rounded-lg transition-all duration-300 max-w-sm py-2 px-3 gap-2 shadow-lg flex items-center text-left ${
                isVisible ? 'opacity-100 translate-x-0' : 'opacity-0 translate-x-full'
            }`}
        >
            <span
                className={`text-base ${TYPE_STYLE[notification.type].className}`}
                aria-hidden="true"
            >
                {TYPE_STYLE[notification.type].icon}
            </span>
            <div className="min-w-0 flex-1">
                <div className="text-sm leading-snug whitespace-normal break-words">
                    {notification.message}
                </div>
            </div>
            {notification.type === 'error' && (
                <button
                    type="button"
                    className="btn btn-ghost btn-xs btn-square"
                    aria-label="Dismiss"
                    onClick={handleRemove}
                >
                    <LuX />
                </button>
            )}
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
