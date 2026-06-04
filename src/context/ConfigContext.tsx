import React, { createContext, useContext, useState, useCallback, ReactNode } from 'react';
import { TauriAPI } from '../tauri';
import { Config } from '../types';

interface ConfigContextType {
    config: Config | null;
    theme: string;
    fuzzyThreshold: number;
    maxResults: number;
    loadConfig: () => Promise<void>;
    saveConfig: (config: Config) => Promise<void>;
}

const ConfigContext = createContext<ConfigContextType | undefined>(undefined);

interface ConfigProviderProps {
    children: ReactNode;
}

export const ConfigProvider: React.FC<ConfigProviderProps> = ({ children }) => {
    const [config, setConfig] = useState<Config | null>(null);
    const [theme, setTheme] = useState('');
    const [fuzzyThreshold, setFuzzyThreshold] = useState(0);
    const [maxResults, setMaxResults] = useState(0);

    const loadConfig = useCallback(async () => {
        try {
            const loaded = await TauriAPI.getConfig();
            setConfig(loaded);
            setTheme(loaded.theme);
            setFuzzyThreshold(loaded.fuzzyThreshold);
            setMaxResults(loaded.maxResults);
        } catch (err) {
            console.warn('Failed to load config:', err);
        }
    }, []);

    const saveConfig = useCallback(async (updated: Config) => {
        await TauriAPI.saveConfig(updated);
    }, []);

    return (
        <ConfigContext.Provider value={{ config, theme, fuzzyThreshold, maxResults, loadConfig, saveConfig }}>
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
