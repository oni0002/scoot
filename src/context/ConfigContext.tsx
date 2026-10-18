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
}

// What Config::default gives in src-tauri/src/config/domain.rs, used until the real config arrives
const DEFAULT_FUZZY_THRESHOLD = 0.5;
const DEFAULT_MAX_RESULTS = 10;

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

    const value = useMemo(
        () => ({
            config,
            theme: config?.theme ?? '',
            fuzzyThreshold: config?.fuzzyThreshold ?? DEFAULT_FUZZY_THRESHOLD,
            maxResults: config?.maxResults ?? DEFAULT_MAX_RESULTS,
            usageHistory: config?.usageHistory ?? true,
            frequentFirst: config?.frequentFirst ?? true,
            loadConfig,
        }),
        [config, loadConfig],
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
