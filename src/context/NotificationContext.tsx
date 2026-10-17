import React, { createContext, useContext, ReactNode } from 'react';
import { useNotifications } from '../hooks/useNotifications';
import { Notification, NotificationToast } from '../components/NotificationToast';
import { useConfigTheme } from './ConfigContext';

interface NotificationContextType {
    showSuccess: (message: string, duration?: number, action?: Notification['action']) => void;
    showError: (message: string, duration?: number, action?: Notification['action']) => void;
    showWarning: (message: string, duration?: number) => void;
    showInfo: (message: string, duration?: number) => void;
}

const NotificationContext = createContext<NotificationContextType | undefined>(undefined);

export const NotificationProvider: React.FC<{ children: ReactNode }> = ({ children }) => {
    const theme = useConfigTheme();
    const { notifications, removeNotification, showSuccess, showError, showWarning, showInfo } =
        useNotifications();

    return (
        <NotificationContext.Provider value={{ showSuccess, showError, showWarning, showInfo }}>
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
