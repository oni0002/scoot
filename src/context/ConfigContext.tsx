import React, { createContext, useContext, useState, useCallback, ReactNode } from 'react';
import { TauriAPI } from '../tauri';

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
    const [theme, setTheme] = useState('');
    const [fuzzyThreshold, setFuzzyThreshold] = useState(0);
    const [maxResults, setMaxResults] = useState(0);

    const loadConfig = useCallback(async () => {
        try {
            const config = await TauriAPI.getConfig();
            setTheme(config.theme);
            setFuzzyThreshold(config.fuzzyThreshold);
            setMaxResults(config.maxResults);
        } catch (err) {
            console.warn('Failed to load config:', err);
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
