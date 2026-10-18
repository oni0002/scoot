import React, { createContext, useContext, useState, useCallback, useMemo, ReactNode } from 'react';
import { TauriAPI } from '../tauri';
import { Config } from '../types';

interface ConfigContextType {
    config: Config | null;
    theme: string;
    fuzzyThreshold: number;
    maxResults: number;
    usageHistory: boolean;
    frequentFirst: boolean;
    loadConfig: () => Promise<void>;
    saveConfig: (config: Config) => Promise<void>;
}

const ConfigContext = createContext<ConfigContextType | undefined>(undefined);

interface ConfigProviderProps {
    children: ReactNode;
}

export const ConfigProvider: React.FC<ConfigProviderProps> = ({ children }) => {
    const [config, setConfig] = useState<Config | null>(null);

    const loadConfig = useCallback(async () => {
        try {
            setConfig(await TauriAPI.getConfig());
        } catch (err) {
            console.warn('Failed to load config:', err);
        }
    }, []);

    const saveConfig = useCallback(async (updated: Config) => {
        await TauriAPI.saveConfig(updated);
    }, []);

    const value = useMemo(
        () => ({
            config,
            theme: config?.theme ?? '',
            fuzzyThreshold: config?.fuzzyThreshold ?? 0,
            maxResults: config?.maxResults ?? 0,
            usageHistory: config?.usageHistory ?? true,
            frequentFirst: config?.frequentFirst ?? true,
            loadConfig,
            saveConfig,
        }),
        [config, loadConfig, saveConfig],
    );

    return <ConfigContext.Provider value={value}>{children}</ConfigContext.Provider>;
};

/** The configured theme, or undefined outside a ConfigProvider (for components that only decorate). */
export const useConfigTheme = (): string | undefined => useContext(ConfigContext)?.theme;

export const useConfigContext = (): ConfigContextType => {
    const context = useContext(ConfigContext);
    if (!context) {
        throw new Error('useConfigContext must be used within a ConfigProvider');
    }
    return context;
};
