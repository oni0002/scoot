import React, { useState, useEffect, useCallback, useRef } from 'react';
import { listen } from '@tauri-apps/api/event';
import { open as openDialog } from '@tauri-apps/plugin-dialog';
import { LuArrowLeft } from 'react-icons/lu';
import { Config } from '../types';
import { useConfigContext } from '../context/ConfigContext';
import { useNotificationContext } from '../context/NotificationContext';
import { getErrorMessage } from '../error';
import { TauriAPI } from '../tauri';
import { THEME_OPTIONS, daisyTheme } from '../theme';
import { ListRow, Row, Section } from './SettingsLayout';

const BROWSERS = ['brave', 'chrome', 'firefox', 'edge'] as const;

interface ConfigScreenProps {
    onBack: () => void;
}

export const ConfigScreen: React.FC<ConfigScreenProps> = ({ onBack }) => {
    const { config, saveConfig } = useConfigContext();
    const { showSuccess } = useNotificationContext();
    const [saveError, setSaveError] = useState<string | null>(null);
    const [edited, setEdited] = useState<Config | null>(null);
    const local = edited ?? config;
    const [capturingHotkey, setCapturingHotkey] = useState(false);
    const hotkeyInputRef = useRef<HTMLInputElement>(null);
    const save = useCallback((updated: Config) => {
        setEdited(updated);
        setSaveError(null);
    }, []);

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

    useEffect(() => {
        const handleKeyDown = (e: KeyboardEvent) => {
            if (e.key === 'Escape' && !capturingHotkey) {
                handleBack();
            }
        };
        window.addEventListener('keydown', handleKeyDown);
        return () => window.removeEventListener('keydown', handleKeyDown);
    }, [handleBack, capturingHotkey]);

    const localRef = useRef(local);
    useEffect(() => {
        localRef.current = local;
    }, [local]);

    // Start/stop hotkey capture mode.
    // Alt+<key> combos are intercepted at the Win32 level (WM_SYSCOMMAND SC_KEYMENU subclass).
    // Other combos (Ctrl+A, Shift+F1, etc.) are handled by handleHotkeyKeyDown in the DOM.
    useEffect(() => {
        if (!capturingHotkey) return;
        let unlisten: (() => void) | null = null;
        void TauriAPI.startHotkeyCapture();
        listen<string>('hotkey-captured', (event) => {
            if (localRef.current) save({ ...localRef.current, hotkey: event.payload });
            setCapturingHotkey(false);
            hotkeyInputRef.current?.blur();
        }).then((fn) => {
            unlisten = fn;
        });
        return () => {
            void TauriAPI.stopHotkeyCapture();
            unlisten?.();
        };
    }, [capturingHotkey, save]);

    const handleHotkeyKeyDown = useCallback(
        (e: React.KeyboardEvent<HTMLInputElement>) => {
            // A plain Tab is never a hotkey (it needs a modifier), so let it move focus on
            if (e.key === 'Tab' && !e.ctrlKey && !e.altKey && !e.shiftKey && !e.metaKey) return;

            e.preventDefault();
            e.stopPropagation();

            if (e.key === 'Escape') {
                setCapturingHotkey(false);
                hotkeyInputRef.current?.blur();
                return;
            }

            const modifiers: string[] = [];
            if (e.ctrlKey) modifiers.push('Ctrl');
            if (e.altKey) modifiers.push('Alt');
            if (e.shiftKey) modifiers.push('Shift');
            if (e.metaKey) modifiers.push('Super');

            const ignoredKeys = ['Control', 'Alt', 'Shift', 'Meta', 'Super'];
            if (ignoredKeys.includes(e.key)) return;

            const keyMap: Record<string, string> = { ' ': 'Space', Enter: 'Enter', Tab: 'Tab' };
            const key = keyMap[e.key] ?? e.key.toUpperCase();

            if (modifiers.length === 0) return;

            const hotkey = [...modifiers, key].join('+');
            setCapturingHotkey(false);
            hotkeyInputRef.current?.blur();
            if (localRef.current) save({ ...localRef.current, hotkey });
        },
        [save],
    );

    const addDirectory = useCallback(async () => {
        if (!local) return;
        const selected = await openDialog({ directory: true, multiple: false });
        if (typeof selected === 'string') {
            save({
                ...local,
                applications: {
                    ...local.applications,
                    directories: [...local.applications.directories, selected],
                },
            });
        }
    }, [local, save]);

    const removeDirectory = useCallback(
        (idx: number) => {
            if (!local) return;
            save({
                ...local,
                applications: {
                    ...local.applications,
                    directories: local.applications.directories.filter((_, i) => i !== idx),
                },
            });
        },
        [local, save],
    );

    const addMarkdownPath = useCallback(async () => {
        if (!local) return;
        const selected = await openDialog({ directory: false, multiple: false });
        if (typeof selected === 'string') {
            save({
                ...local,
                markdown: { ...local.markdown, paths: [...local.markdown.paths, selected] },
            });
        }
    }, [local, save]);

    const removeMarkdownPath = useCallback(
        (idx: number) => {
            if (!local) return;
            save({
                ...local,
                markdown: {
                    ...local.markdown,
                    paths: local.markdown.paths.filter((_, i) => i !== idx),
                },
            });
        },
        [local, save],
    );

    const [newExtension, setNewExtension] = useState('');
    const [addingExtension, setAddingExtension] = useState(false);

    const confirmAddExtension = useCallback(() => {
        if (!local || !newExtension.trim()) {
            setAddingExtension(false);
            return;
        }
        const ext = newExtension.trim().replace(/^\./, '');
        if (!local.applications.extensions.includes(ext)) {
            save({
                ...local,
                applications: {
                    ...local.applications,
                    extensions: [...local.applications.extensions, ext],
                },
            });
        }
        setAddingExtension(false);
        setNewExtension('');
    }, [local, newExtension, save]);

    const cancelAddExtension = useCallback(() => {
        setAddingExtension(false);
        setNewExtension('');
    }, []);

    const removeExtension = useCallback(
        (idx: number) => {
            if (!local) return;
            save({
                ...local,
                applications: {
                    ...local.applications,
                    extensions: local.applications.extensions.filter((_, i) => i !== idx),
                },
            });
        },
        [local, save],
    );

    const removeIgnored = useCallback(
        (idx: number) => {
            if (!local) return;
            save({ ...local, ignored: local.ignored.filter((_, i) => i !== idx) });
        },
        [local, save],
    );

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
                <Section title="General">
                    <Row label="Theme" htmlFor="cfg-theme">
                        <select
                            id="cfg-theme"
                            className="select select-sm w-40"
                            value={local.theme}
                            onChange={(e) => save({ ...local, theme: e.target.value })}
                        >
                            {THEME_OPTIONS.map((t) => (
                                <option key={t.value} value={t.value}>
                                    {t.label}
                                </option>
                            ))}
                        </select>
                    </Row>
                    <Row
                        label="Hotkey"
                        htmlFor="cfg-hotkey"
                        description="Click, then press a key combination. Esc cancels."
                    >
                        <input
                            id="cfg-hotkey"
                            ref={hotkeyInputRef}
                            className="input input-sm w-40 text-center cursor-pointer"
                            value={capturingHotkey ? 'Press a key...' : local.hotkey}
                            readOnly
                            onFocus={() => setCapturingHotkey(true)}
                            onBlur={() => setCapturingHotkey(false)}
                            onKeyDown={handleHotkeyKeyDown}
                        />
                    </Row>
                    <Row
                        label="Max results"
                        htmlFor="cfg-max-results"
                        description="How many results to show."
                    >
                        <input
                            id="cfg-max-results"
                            type="number"
                            min={1}
                            max={100}
                            className="input input-sm w-24"
                            value={local.maxResults}
                            onChange={(e) =>
                                save({
                                    ...local,
                                    maxResults: Math.max(1, Math.min(100, Number(e.target.value))),
                                })
                            }
                        />
                    </Row>
                    <Row
                        label="Fuzzy threshold"
                        htmlFor="cfg-fuzzy"
                        description="Higher is stricter, lower is more permissive."
                    >
                        <div className="flex items-center gap-3">
                            <input
                                id="cfg-fuzzy"
                                type="range"
                                min={0}
                                max={1}
                                step={0.1}
                                className="range w-44"
                                value={local.fuzzyThreshold}
                                onChange={(e) =>
                                    save({ ...local, fuzzyThreshold: Number(e.target.value) })
                                }
                            />
                            <span className="w-6 text-right text-sm tabular-nums">
                                {local.fuzzyThreshold.toFixed(1)}
                            </span>
                        </div>
                    </Row>
                    <Row
                        label="Reload interval"
                        htmlFor="cfg-reload"
                        description="Minutes between source reloads."
                    >
                        <input
                            id="cfg-reload"
                            type="number"
                            min={1}
                            className="input input-sm w-24"
                            value={local.reloadIntervalMinutes}
                            onChange={(e) =>
                                save({
                                    ...local,
                                    reloadIntervalMinutes: Math.max(1, Number(e.target.value)),
                                })
                            }
                        />
                    </Row>
                </Section>

                <Section
                    title="Bookmarks"
                    enabled={{
                        value: local.bookmarks.enabled,
                        onChange: (enabled) =>
                            save({ ...local, bookmarks: { ...local.bookmarks, enabled } }),
                    }}
                >
                    <Row label="Browser" htmlFor="cfg-browser">
                        <select
                            id="cfg-browser"
                            className="select select-sm w-40"
                            value={local.bookmarks.browser}
                            onChange={(e) =>
                                save({
                                    ...local,
                                    bookmarks: {
                                        ...local.bookmarks,
                                        browser: e.target.value as (typeof BROWSERS)[number],
                                    },
                                })
                            }
                        >
                            {BROWSERS.map((b) => (
                                <option key={b} value={b}>
                                    {b}
                                </option>
                            ))}
                        </select>
                    </Row>
                </Section>

                <Section
                    title="Applications"
                    enabled={{
                        value: local.applications.enabled,
                        onChange: (enabled) =>
                            save({ ...local, applications: { ...local.applications, enabled } }),
                    }}
                >
                    <ListRow
                        label="Directories"
                        description="Folders scanned for application shortcuts."
                        items={local.applications.directories}
                        onRemove={removeDirectory}
                        addLabel="Add directory"
                        onAdd={addDirectory}
                    />
                    <ListRow
                        label="Extensions"
                        description="File types treated as applications."
                        items={local.applications.extensions.map((ext) => `.${ext}`)}
                        onRemove={removeExtension}
                        addLabel="Add extension"
                        onAdd={() => setAddingExtension(true)}
                        addDisabled={addingExtension}
                        extra={
                            addingExtension && (
                                <li className="flex items-center gap-2 rounded-md bg-base-100 py-1 pl-2 pr-1">
                                    <input
                                        autoFocus
                                        autoComplete="off"
                                        className="input input-xs flex-1 min-w-0 bg-transparent"
                                        placeholder="e.g. lnk"
                                        aria-label="New extension"
                                        value={newExtension}
                                        onChange={(e) => setNewExtension(e.target.value)}
                                        onKeyDown={(e) => {
                                            if (e.key === 'Enter') confirmAddExtension();
                                            if (e.key === 'Escape') cancelAddExtension();
                                        }}
                                    />
                                    <button
                                        type="button"
                                        className="btn btn-ghost btn-xs"
                                        onClick={confirmAddExtension}
                                    >
                                        OK
                                    </button>
                                    <button
                                        type="button"
                                        className="btn btn-ghost btn-xs"
                                        onClick={cancelAddExtension}
                                    >
                                        Cancel
                                    </button>
                                </li>
                            )
                        }
                    />
                </Section>

                <Section
                    title="Markdown"
                    enabled={{
                        value: local.markdown.enabled,
                        onChange: (enabled) =>
                            save({ ...local, markdown: { ...local.markdown, enabled } }),
                    }}
                >
                    <ListRow
                        label="Files"
                        description="Markdown files to read links from."
                        items={local.markdown.paths}
                        onRemove={removeMarkdownPath}
                        addLabel="Add file"
                        onAdd={addMarkdownPath}
                    />
                </Section>

                <Section title="Ignored">
                    <ListRow
                        label="Hidden from search"
                        description="Add from the ⋮ menu on a result."
                        items={local.ignored}
                        onRemove={removeIgnored}
                        emptyText="No ignored commands."
                    />
                </Section>
            </div>
        </div>
    );
};
