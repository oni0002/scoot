import React, { createContext, useContext, useState, useCallback, useMemo, ReactNode } from 'react';
import { TauriAPI } from '../tauri';
import { Settings } from '../types';

interface SettingsContextType {
    settings: Settings | null;
    theme: string;
    fuzzyThreshold: number;
    maxResults: number;
    usageHistory: boolean;
    frequentFirst: boolean;
    loadSettings: () => Promise<void>;
}

// What Settings::default gives in src-tauri/src/settings/domain.rs, used until the real settings arrives
const DEFAULT_FUZZY_THRESHOLD = 0.5;
const DEFAULT_MAX_RESULTS = 10;

const SettingsContext = createContext<SettingsContextType | undefined>(undefined);

interface SettingsProviderProps {
    children: ReactNode;
}

export const SettingsProvider: React.FC<SettingsProviderProps> = ({ children }) => {
    const [settings, setSettings] = useState<Settings | null>(null);

    const loadSettings = useCallback(async () => {
        try {
            setSettings(await TauriAPI.getSettings());
        } catch (err) {
            console.warn('Failed to load settings:', err);
        }
    }, []);

    const value = useMemo(
        () => ({
            settings,
            theme: settings?.theme ?? '',
            fuzzyThreshold: settings?.fuzzyThreshold ?? DEFAULT_FUZZY_THRESHOLD,
            maxResults: settings?.maxResults ?? DEFAULT_MAX_RESULTS,
            usageHistory: settings?.usageHistory ?? true,
            frequentFirst: settings?.frequentFirst ?? true,
            loadSettings,
        }),
        [settings, loadSettings],
    );

    return <SettingsContext.Provider value={value}>{children}</SettingsContext.Provider>;
};

/** The configured theme, or undefined outside a SettingsProvider (for components that only decorate). */
export const useSettingsTheme = (): string | undefined => useContext(SettingsContext)?.theme;

export const useSettingsContext = (): SettingsContextType => {
    const context = useContext(SettingsContext);
    if (!context) {
        throw new Error('useSettingsContext must be used within a SettingsProvider');
    }
    return context;
};
