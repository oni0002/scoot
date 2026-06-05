import React, { useState, useEffect, useCallback, useRef } from 'react';
import { listen } from '@tauri-apps/api/event';
import { open as openDialog } from '@tauri-apps/plugin-dialog';
import { LuArrowLeft, LuX } from 'react-icons/lu';
import { Config } from '../types';
import { useConfigContext } from '../context/ConfigContext';
import { TauriAPI } from '../tauri';

const THEMES = [
    'light', 'dark', 'cupcake', 'bumblebee', 'emerald', 'corporate', 'synthwave',
    'retro', 'cyberpunk', 'valentine', 'halloween', 'garden', 'forest', 'aqua',
    'lofi', 'pastel', 'fantasy', 'wireframe', 'black', 'luxury', 'dracula',
    'cmyk', 'autumn', 'business', 'acid', 'lemonade', 'night', 'coffee', 'winter',
    'dim', 'nord', 'sunset',
];

const BROWSERS = ['brave', 'chrome', 'firefox', 'edge'] as const;

interface ConfigScreenProps {
    onBack: () => void;
}

export const ConfigScreen: React.FC<ConfigScreenProps> = ({ onBack }) => {
    const { config, saveConfig } = useConfigContext();
    const [local, setLocal] = useState<Config | null>(null);
    const [capturingHotkey, setCapturingHotkey] = useState(false);
    const hotkeyInputRef = useRef<HTMLInputElement>(null);
    useEffect(() => {
        if (config) setLocal(prev => prev ?? { ...config });
    }, [config]);

    const save = useCallback((updated: Config) => {
        setLocal(updated);
    }, []);

    const handleBack = useCallback(async () => {
        if (local) {
            try {
                await saveConfig(local);
            } catch (err) {
                console.error('Failed to save config:', err);
            }
        }
        onBack();
    }, [local, saveConfig, onBack]);

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
    useEffect(() => { localRef.current = local; }, [local]);

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
        }).then(fn => { unlisten = fn; });
        return () => {
            void TauriAPI.stopHotkeyCapture();
            unlisten?.();
        };
    }, [capturingHotkey, save]);

    const handleHotkeyKeyDown = useCallback((e: React.KeyboardEvent<HTMLInputElement>) => {
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

        const keyMap: Record<string, string> = { ' ': 'Space', 'Enter': 'Enter', 'Tab': 'Tab' };
        const key = keyMap[e.key] ?? e.key.toUpperCase();

        if (modifiers.length === 0) return;

        const hotkey = [...modifiers, key].join('+');
        setCapturingHotkey(false);
        hotkeyInputRef.current?.blur();
        if (localRef.current) save({ ...localRef.current, hotkey });
    }, [save]);

    const addDirectory = useCallback(async () => {
        if (!local) return;
        const selected = await openDialog({ directory: true, multiple: false });
        if (typeof selected === 'string') {
            save({ ...local, applications: { ...local.applications, directories: [...local.applications.directories, selected] } });
        }
    }, [local, save]);

    const removeDirectory = useCallback((idx: number) => {
        if (!local) return;
        save({ ...local, applications: { ...local.applications, directories: local.applications.directories.filter((_, i) => i !== idx) } });
    }, [local, save]);

    const addMarkdownPath = useCallback(async () => {
        if (!local) return;
        const selected = await openDialog({ directory: false, multiple: false });
        if (typeof selected === 'string') {
            save({ ...local, markdown: { ...local.markdown, paths: [...local.markdown.paths, selected] } });
        }
    }, [local, save]);

    const removeMarkdownPath = useCallback((idx: number) => {
        if (!local) return;
        save({ ...local, markdown: { ...local.markdown, paths: local.markdown.paths.filter((_, i) => i !== idx) } });
    }, [local, save]);

    const [newExtension, setNewExtension] = useState('');
    const [addingExtension, setAddingExtension] = useState(false);

    const confirmAddExtension = useCallback(() => {
        if (!local || !newExtension.trim()) { setAddingExtension(false); return; }
        const ext = newExtension.trim().replace(/^\./, '');
        if (!local.applications.extensions.includes(ext)) {
            save({ ...local, applications: { ...local.applications, extensions: [...local.applications.extensions, ext] } });
        }
        setAddingExtension(false);
        setNewExtension('');
    }, [local, newExtension, save]);

    const cancelAddExtension = useCallback(() => {
        setAddingExtension(false);
        setNewExtension('');
    }, []);

    const removeExtension = useCallback((idx: number) => {
        if (!local) return;
        save({ ...local, applications: { ...local.applications, extensions: local.applications.extensions.filter((_, i) => i !== idx) } });
    }, [local, save]);

    const removeIgnored = useCallback((idx: number) => {
        if (!local) return;
        save({ ...local, ignored: local.ignored.filter((_, i) => i !== idx) });
    }, [local, save]);

    if (!local) return null;

    return (
        <div className="h-screen w-screen flex flex-col bg-base-100 text-base-content scrollbar-gutter-auto" data-theme={local.theme}>
            {/* Header */}
            <div className="flex-none p-4 flex items-center gap-2">
                <button className="btn btn-ghost btn-sm btn-square" onClick={handleBack}>
                    <LuArrowLeft />
                </button>
                <h3>Settings</h3>
            </div>

            {/* Body */}
            <div className="flex-1 overflow-y-auto px-4 pb-4 scrollbar-gutter-auto space-y-4">
                {/* General */}
                <div className="divider divider-start text-sm pb-4">General</div>
                {/* Theme */}
                <div className="form-control w-full">
                    <label className="label"><span className="label-text">Theme</span></label>
                    <select
                        className="select select-bordered select-sm w-48"
                        value={local.theme}
                        onChange={e => save({ ...local, theme: e.target.value })}
                    >
                        {THEMES.map(t => <option key={t} value={t}>{t}</option>)}
                    </select>
                </div>
                {/* Hotkey */}
                <div className="form-control w-full">
                    <label className="label"><span className="label-text">Hotkey</span></label>
                    <p className="text-xs text-base-content/50 mb-1">Click to capture. Press a key combo (e.g. Alt+Space) to set.</p>
                    <input
                        ref={hotkeyInputRef}
                        className="input input-bordered input-sm w-48 text-center cursor-pointer"
                        value={capturingHotkey ? 'Press a key...' : local.hotkey}
                        readOnly
                        onFocus={() => setCapturingHotkey(true)}
                        onBlur={() => setCapturingHotkey(false)}
                        onKeyDown={handleHotkeyKeyDown}
                    />
                </div>
                {/* Max Results */}
                <div className="form-control w-full">
                    <label className="label"><span className="label-text">Max Results</span></label>
                    <p className="text-xs text-base-content/50 mb-1">Maximum number of search results to show.</p>
                    <input
                        type="number"
                        min={1}
                        max={100}
                        className="input input-bordered input-sm w-24"
                        value={local.maxResults}
                        onChange={e => save({ ...local, maxResults: Math.max(1, Math.min(100, Number(e.target.value))) })}
                    />
                </div>
                {/* Fuzzy Threshold */}
                <div className="form-control w-full">
                    <label className="label">
                        <span className="label-text">Fuzzy Threshold</span>
                    </label>
                    <p className="text-xs text-base-content/50 mb-1">How strictly queries must match. Higher = stricter. Lower = more permissive.</p>
                    <div className="flex items-center gap-2">
                        <input
                            type="range"
                            min={0}
                            max={1}
                            step={0.1}
                            className="range range-sm w-64"
                            value={local.fuzzyThreshold}
                            onChange={e => save({ ...local, fuzzyThreshold: Number(e.target.value) })}
                        />
                        <span className="text-xs w-6 text-right tabular-nums">{local.fuzzyThreshold.toFixed(1)}</span>
                    </div>
                </div>
                {/* Reload Interval */}
                <div className="form-control w-full">
                    <label className="label"><span className="label-text">Reload Interval (min)</span></label>
                    <p className="text-xs text-base-content/50 mb-1">How often bookmarks and applications are reloaded (in minutes).</p>
                    <input
                        type="number"
                        min={1}
                        className="input input-bordered input-sm w-24"
                        value={local.reloadIntervalMinutes}
                        onChange={e => save({ ...local, reloadIntervalMinutes: Math.max(1, Number(e.target.value)) })}
                    />
                </div>
                {/* Ignored */}
                <div className="form-control w-full">
                    <label className="label"><span className="label-text">Ignored</span></label>
                    <p className="text-xs text-base-content/50 mb-1">Commands hidden from search results. Add via the ⋯ menu on a search result.</p>
                    {local.ignored.length === 0 ? (
                        <p className="text-xs text-base-content/40 px-1">No ignored commands.</p>
                    ) : (
                        <ul className="space-y-1">
                            {local.ignored.map((cmd, i) => (
                                <li key={i} className="flex items-center gap-2 text-xs bg-base-200 rounded px-2 py-1">
                                    <span className="flex-1 truncate">{cmd}</span>
                                    <button className="btn btn-ghost btn-xs p-0" onClick={() => removeIgnored(i)}><LuX size={12} /></button>
                                </li>
                            ))}
                        </ul>
                    )}
                </div>

                {/* Bookmarks */}
                <div className="divider divider-start text-sm pt-6 pb-4">Bookmarks</div>
                {/* Enable button */}
                <div className="form-control">
                    <label className="label cursor-pointer justify-start gap-4">
                        <span className="label-text">Enabled</span>
                        <input
                            type="checkbox"
                            className="toggle toggle-sm"
                            checked={local.bookmarks.enabled}
                            onChange={e => save({ ...local, bookmarks: { ...local.bookmarks, enabled: e.target.checked } })}
                        />
                    </label>
                </div>
                {/* Browser */}
                <div className="form-control w-full">
                    <label className="label"><span className="label-text">Browser</span></label>
                    <select
                        className="select select-bordered select-sm w-48"
                        value={local.bookmarks.browser}
                        onChange={e => save({ ...local, bookmarks: { ...local.bookmarks, browser: e.target.value as typeof BROWSERS[number] } })}
                    >
                        {BROWSERS.map(b => <option key={b} value={b}>{b}</option>)}
                    </select>
                </div>

                {/* Applications */}
                <div className="divider divider-start text-sm pt-6 pb-4">Applications</div>
                {/* Enable button */}
                <div className="form-control">
                    <label className="label cursor-pointer justify-start gap-4">
                        <span className="label-text">Enabled</span>
                        <input
                            type="checkbox"
                            className="toggle toggle-sm"
                            checked={local.applications.enabled}
                            onChange={e => save({ ...local, applications: { ...local.applications, enabled: e.target.checked } })}
                        />
                    </label>
                </div>
                {/* Directories */}
                <div className="form-control w-full">
                    <label className="label">
                        <span className="label-text">Directories</span>
                    </label>
                    <p className="text-xs text-base-content/50 mb-1">Folders to scan for application shortcuts (.lnk files by default).</p>
                    <ul className="space-y-1">
                        {local.applications.directories.map((dir, i) => (
                            <li key={i} className="flex items-center gap-2 text-xs bg-base-200 rounded pl-2 pr-1 py-1">
                                <span className="flex-1 truncate">{dir}</span>
                                <button className="btn btn-ghost btn-xs btn-square" onClick={() => removeDirectory(i)}><LuX size={12} /></button>
                            </li>
                        ))}
                    </ul>
                    <button className="btn btn-neutral btn-xs mt-1 self-start" onClick={addDirectory}>Add</button>
                </div>
                {/* Extensions */}
                <div className="form-control w-full">
                    <label className="label">
                        <span className="label-text">Extensions</span>
                    </label>
                    <p className="text-xs text-base-content/50 mb-1">File extensions treated as applications when scanning directories.</p>
                    <ul className="space-y-1">
                        {local.applications.extensions.map((ext, i) => (
                            <li key={i} className="flex items-center gap-2 text-xs bg-base-200 rounded pl-2 pr-1 py-1">
                                <span className="flex-1 truncate">.{ext}</span>
                                <button className="btn btn-ghost btn-xs btn-square" onClick={() => removeExtension(i)}><LuX size={12} /></button>
                            </li>
                        ))}
                        {addingExtension && (
                            <li className="flex items-center gap-2 text-xs bg-base-200 rounded pl-2 pr-1 py-1">
                                <input
                                    autoFocus
                                    className="input input-xs flex-1 min-w-0 bg-transparent"
                                    placeholder="e.g. lnk"
                                    value={newExtension}
                                    onChange={e => setNewExtension(e.target.value)}
                                    onKeyDown={e => { if (e.key === 'Enter') confirmAddExtension(); if (e.key === 'Escape') cancelAddExtension(); }}
                                />
                                <button className="btn btn-ghost btn-xs" onClick={confirmAddExtension}>OK</button>
                                <button className="btn btn-ghost btn-xs" onClick={cancelAddExtension}>Cancel</button>
                            </li>
                        )}
                    </ul>
                    <button className="btn btn-neutral btn-xs mt-1 self-start" onClick={() => setAddingExtension(true)} disabled={addingExtension}>Add</button>
                </div>

                {/* Markdown */}
                <div className="divider divider-start text-sm pt-6 pb-4">Markdown</div>
                {/* Enable button */}
                <div className="form-control">
                    <label className="label cursor-pointer justify-start gap-4">
                        <span className="label-text">Enabled</span>
                        <input
                            type="checkbox"
                            className="toggle toggle-sm"
                            checked={local.markdown.enabled}
                            onChange={e => save({ ...local, markdown: { ...local.markdown, enabled: e.target.checked } })}
                        />
                    </label>
                </div>
                {/* Files */}
                <div className="form-control w-full">
                    <label className="label">
                        <span className="label-text">Files</span>
                    </label>
                    <p className="text-xs text-base-content/50 mb-1">Markdown files to extract URL and file-path links from.</p>
                    <ul className="space-y-1">
                        {local.markdown.paths.map((p, i) => (
                            <li key={i} className="flex items-center gap-2 text-xs bg-base-200 rounded px-2 py-1">
                                <span className="flex-1 truncate">{p}</span>
                                <button className="btn btn-ghost btn-xs p-0" onClick={() => removeMarkdownPath(i)}><LuX size={12} /></button>
                            </li>
                        ))}
                    </ul>
                    <button className="btn btn-neutral btn-xs mt-1 self-start" onClick={addMarkdownPath}>Add</button>
                </div>
            </div>
        </div>
    );
};
