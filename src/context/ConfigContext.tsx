import React, { createContext, useContext, useState, useCallback, ReactNode } from 'react';
import { TauriAPI } from '../api/tauri';

interface ConfigContextType {
    theme: string;
    fuzzyThreshold: number;
    maxResults: number;
    loadConfig: () => Promise<void>;
}

const ConfigContext = createContext<ConfigContextType | undefined>(undefined);

interface ConfigProviderProps {
    children: ReactNode;
}

export const ConfigProvider: React.FC<ConfigProviderProps> = ({ children }) => {
    const [theme, setTheme] = useState('dark');
    const [fuzzyThreshold, setFuzzyThreshold] = useState(0.5);
    const [maxResults, setMaxResults] = useState(10);

    const loadConfig = useCallback(async () => {
        try {
            const config = await TauriAPI.getConfig();
            setTheme(config.theme || 'dark');
            setFuzzyThreshold(config.fuzzyThreshold || 0.5);
            setMaxResults(config.maxResults || 10);
        } catch (err) {
            console.warn('Failed to load config, using default values:', err);
        }
    }, []);

    return (
        <ConfigContext.Provider value={{ theme, fuzzyThreshold, maxResults, loadConfig }}>
            {children}
        </ConfigContext.Provider>
    );
};

export const useConfigContext = (): ConfigContextType => {
    const context = useContext(ConfigContext);
    if (!context) {
        throw new Error('useConfigContext must be used within a ConfigProvider');
    }
    return context;
};
