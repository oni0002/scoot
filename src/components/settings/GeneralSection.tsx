import React, { useEffect, useState } from 'react';
import { TauriAPI } from '../../tauri';
import { Settings } from '../../types';
import { THEME_OPTIONS } from '../../theme';
import { HotkeyCapture } from '../../hooks/useHotkeyCapture';
import { useNotificationContext } from '../../context/NotificationContext';
import { ConfirmDialog } from '../ConfirmDialog';
import { Row, Section } from '../SettingsLayout';
import { NumberInput } from './NumberInput';

interface GeneralSectionProps {
    settings: Settings;
    onChange: (settings: Settings) => void;
    hotkey: HotkeyCapture;
    onClearHistory: () => Promise<void>;
}

export const GeneralSection: React.FC<GeneralSectionProps> = ({
    settings,
    onChange,
    hotkey,
    onClearHistory,
}) => {
    const { showSuccess, reportError } = useNotificationContext();
    const [confirmingClear, setConfirmingClear] = useState(false);
    // null until the Startup folder has been checked
    const [startupShortcut, setStartupShortcut] = useState<boolean | null>(null);
    const [startupBusy, setStartupBusy] = useState(false);

    useEffect(() => {
        TauriAPI.getStartupShortcut()
            .then((exists) => setStartupShortcut(Boolean(exists)))
            .catch(() => setStartupShortcut(false));
    }, []);

    // Like clearing history, this takes effect at once, not when the screen closes
    const toggleStartupShortcut = async () => {
        if (startupShortcut === null) return;
        setStartupBusy(true);
        try {
            const created = await TauriAPI.setStartupShortcut(!startupShortcut);
            setStartupShortcut(created);
            showSuccess(created ? 'Scoot will run at startup' : 'Scoot will not run at startup');
        } catch (err) {
            reportError(`${startupShortcut ? 'remove' : 'create'} the startup shortcut`, err);
        } finally {
            setStartupBusy(false);
        }
    };

    // Clearing takes effect at once, not with the other settings when the screen closes
    const clearHistory = async () => {
        try {
            await onClearHistory();
            showSuccess('History cleared');
        } catch (err) {
            reportError('clear history', err);
        }
    };

    return (
        <>
            <Section title="General">
                <Row label="Theme" htmlFor="cfg-theme">
                    <select
                        id="cfg-theme"
                        className="select select-sm w-40"
                        value={settings.theme}
                        onChange={(e) => onChange({ ...settings, theme: e.target.value })}
                    >
                        {THEME_OPTIONS.map((t) => (
                            <option key={t.value} value={t.value}>
                                {t.label}
                            </option>
                        ))}
                    </select>
                </Row>
                <Row
                    label="Run at startup"
                    htmlFor="cfg-startup"
                    description="Adds a shortcut to the Startup folder. Turn it off and on again if you move Scoot."
                >
                    <input
                        id="cfg-startup"
                        type="checkbox"
                        className="toggle"
                        checked={startupShortcut === true}
                        disabled={startupShortcut === null || startupBusy}
                        onChange={toggleStartupShortcut}
                    />
                </Row>
                <Row
                    label="Hotkey"
                    htmlFor="cfg-hotkey"
                    description="Click, then press a key combination. Esc cancels."
                >
                    <input
                        id="cfg-hotkey"
                        className="input input-sm w-40 text-center cursor-pointer"
                        value={hotkey.capturing ? 'Press a key...' : settings.hotkey}
                        readOnly
                        {...hotkey.inputProps}
                    />
                </Row>
                <Row
                    label="Max results"
                    htmlFor="cfg-max-results"
                    description="How many results to show."
                >
                    <NumberInput
                        id="cfg-max-results"
                        min={1}
                        max={100}
                        value={settings.maxResults}
                        onChange={(maxResults) => onChange({ ...settings, maxResults })}
                    />
                </Row>
                <Row
                    label="Fuzzy threshold"
                    htmlFor="cfg-fuzzy"
                    description="Lower matches more exactly, higher is more permissive."
                >
                    <div className="flex w-40 items-center gap-2">
                        <input
                            id="cfg-fuzzy"
                            type="range"
                            min={0}
                            max={1}
                            step={0.1}
                            className="range range-sm min-w-0 flex-1"
                            value={settings.fuzzyThreshold}
                            onChange={(e) =>
                                onChange({ ...settings, fuzzyThreshold: Number(e.target.value) })
                            }
                        />
                        <span className="w-6 text-right text-sm tabular-nums">
                            {settings.fuzzyThreshold.toFixed(1)}
                        </span>
                    </div>
                </Row>
                <Row
                    label="Reload interval"
                    htmlFor="cfg-reload"
                    description="Minutes between source reloads."
                >
                    <NumberInput
                        id="cfg-reload"
                        min={1}
                        max={1440}
                        value={settings.reloadIntervalMinutes}
                        onChange={(reloadIntervalMinutes) =>
                            onChange({ ...settings, reloadIntervalMinutes })
                        }
                    />
                </Row>
                <Row
                    label="Usage history"
                    htmlFor="cfg-usage-history"
                    description="Rank commands you run often and recently higher."
                >
                    <input
                        id="cfg-usage-history"
                        type="checkbox"
                        className="toggle"
                        checked={settings.usageHistory}
                        onChange={(e) => onChange({ ...settings, usageHistory: e.target.checked })}
                    />
                </Row>
                <Row
                    label="Frequent commands"
                    htmlFor="cfg-frequent-first"
                    description={
                        settings.usageHistory
                            ? 'List the commands you run most before you type.'
                            : 'Needs usage history.'
                    }
                >
                    <input
                        id="cfg-frequent-first"
                        type="checkbox"
                        className="toggle"
                        disabled={!settings.usageHistory}
                        checked={settings.frequentFirst && settings.usageHistory}
                        onChange={(e) => onChange({ ...settings, frequentFirst: e.target.checked })}
                    />
                </Row>
                <Row label="Clear history" description="Forget which commands you have run.">
                    <button
                        type="button"
                        className="btn btn-sm btn-outline btn-error"
                        onClick={() => setConfirmingClear(true)}
                    >
                        Clear
                    </button>
                </Row>
            </Section>
            <ConfirmDialog
                isOpen={confirmingClear}
                title="Clear history?"
                confirmLabel="Clear"
                destructive
                onConfirm={clearHistory}
                onCancel={() => setConfirmingClear(false)}
            >
                Search results go back to ordering by match only. This cannot be undone.
            </ConfirmDialog>
        </>
    );
};
