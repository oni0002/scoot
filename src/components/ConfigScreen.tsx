import React, { useState, useEffect, useCallback } from 'react';
import { LuArrowLeft } from 'react-icons/lu';
import { Config } from '../types';
import { useConfigContext } from '../context/ConfigContext';
import { useNotificationContext } from '../context/NotificationContext';
import { useHotkeyCapture } from '../hooks/useHotkeyCapture';
import { getErrorMessage } from '../error';
import { daisyTheme } from '../theme';
import { GeneralSection } from './settings/GeneralSection';
import { BookmarksSection } from './settings/BookmarksSection';
import { ApplicationsSection } from './settings/ApplicationsSection';
import { MarkdownSection } from './settings/MarkdownSection';
import { IgnoredSection } from './settings/IgnoredSection';

interface ConfigScreenProps {
    onClearHistory: () => Promise<void>;
    onBack: () => void;
}

export const ConfigScreen: React.FC<ConfigScreenProps> = ({ onBack, onClearHistory }) => {
    const { config, saveConfig } = useConfigContext();
    const { showSuccess } = useNotificationContext();
    const [saveError, setSaveError] = useState<string | null>(null);
    const [edited, setEdited] = useState<Config | null>(null);
    const local = edited ?? config;

    const change = useCallback((updated: Config) => {
        setEdited(updated);
        setSaveError(null);
    }, []);

    const hotkey = useHotkeyCapture((captured) => {
        if (local) change({ ...local, hotkey: captured });
    });

    // Settings are written when the screen closes. A failed write keeps the screen open, so the
    // edits are not lost behind a screen that looks saved.
    const handleBack = useCallback(async () => {
        if (!edited || JSON.stringify(edited) === JSON.stringify(config)) {
            onBack();
            return;
        }
        try {
            await saveConfig(edited);
        } catch (err) {
            setSaveError(getErrorMessage(err));
            return;
        }
        showSuccess('Settings saved');
        onBack();
    }, [edited, config, saveConfig, showSuccess, onBack]);

    // While the hotkey field is capturing, Esc cancels the capture instead of closing the screen
    const capturing = hotkey.capturing;
    useEffect(() => {
        const handleKeyDown = (e: KeyboardEvent) => {
            if (e.key === 'Escape' && !capturing) {
                handleBack();
            }
        };
        window.addEventListener('keydown', handleKeyDown);
        return () => window.removeEventListener('keydown', handleKeyDown);
    }, [handleBack, capturing]);

    if (!local) return null;

    return (
        <div
            className="h-screen w-screen flex flex-col pb-4 bg-base-100 text-base-content scrollbar-gutter-auto"
            data-theme={daisyTheme(local.theme)}
        >
            {/* Header */}
            <div className="flex-none p-4 pb-2 flex items-center gap-2">
                <button
                    className="btn btn-ghost btn-sm btn-square"
                    onClick={handleBack}
                    aria-label="Back"
                    title="Back"
                >
                    <LuArrowLeft />
                </button>
                <h3>Settings</h3>
            </div>

            {saveError && (
                <div
                    role="alert"
                    className="mx-4 flex flex-none items-center gap-3 rounded-lg border border-error p-2 text-sm text-error"
                >
                    <span className="min-w-0 flex-1 break-words">
                        Could not save settings: {saveError}
                    </span>
                    <button type="button" className="btn btn-ghost btn-xs" onClick={onBack}>
                        Discard and close
                    </button>
                </div>
            )}

            {/* Body */}
            <div className="flex-1 overflow-y-auto px-4 pb-2 scrollbar-gutter-auto">
                {/* Same left edge as the section headings; -mb-3 tightens the gap to the first one */}
                <p className="-mb-3 mt-1 text-xs text-subtle">
                    Changes are saved when you go back.
                </p>
                <GeneralSection
                    config={local}
                    onChange={change}
                    hotkey={hotkey}
                    onClearHistory={onClearHistory}
                />
                <BookmarksSection config={local} onChange={change} />
                <ApplicationsSection config={local} onChange={change} />
                <MarkdownSection config={local} onChange={change} />
                <IgnoredSection config={local} onChange={change} />
            </div>
        </div>
    );
};
