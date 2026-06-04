import React, { useState, useEffect, useCallback, useRef } from 'react';
import { listen } from '@tauri-apps/api/event';
import { open as openDialog } from '@tauri-apps/plugin-dialog';
import { LuArrowLeft, LuPlus, LuX } from 'react-icons/lu';
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

    // Start/stop OS-level keyboard hook during hotkey capture.
    // WH_KEYBOARD_LL is needed because WM_SYSKEYDOWN (Alt+<key>) never reaches
    // the WebView2 DOM — Windows consumes it before forwarding to the renderer.
    useEffect(() => {
        if (!capturingHotkey) return;
        let unlisten: (() => void) | null = null;
        TauriAPI.startHotkeyCapture();
        listen<string>('hotkey-captured', (event) => {
            if (localRef.current) save({ ...localRef.current, hotkey: event.payload });
            setCapturingHotkey(false);
            hotkeyInputRef.current?.blur();
        }).then(fn => { unlisten = fn; });
        return () => {
            TauriAPI.stopHotkeyCapture();
            unlisten?.();
        };
    }, [capturingHotkey, save]);

    const handleHotkeyKeyDown = useCallback((e: React.KeyboardEvent<HTMLInputElement>) => {
        e.preventDefault();
        e.stopPropagation();
        if (e.key === 'Escape') {
            setCapturingHotkey(false);
            hotkeyInputRef.current?.blur();
        }
    }, []);

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

    const addExtension = useCallback(() => {
        if (!local || !newExtension.trim()) return;
        const ext = newExtension.trim().replace(/^\./, '');
        if (local.applications.extensions.includes(ext)) return;
        save({ ...local, applications: { ...local.applications, extensions: [...local.applications.extensions, ext] } });
        setNewExtension('');
    }, [local, newExtension, save]);

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
            <div className="flex-1 overflow-y-auto px-4 pb-4 scrollbar-gutter-auto space-y-2">
                {/* General */}
                <h2 className="text-xs font-bold uppercase tracking-wider text-base-content/50 mt-2">General</h2>
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
                <div className="form-control w-64">
                    <label className="label">
                        <span className="label-text">Fuzzy Threshold</span>
                        <span className="label-text-alt">{local.fuzzyThreshold.toFixed(2)}</span>
                    </label>
                    <input
                        type="range"
                        min={0}
                        max={1}
                        step={0.1}
                        className="range range-sm"
                        value={local.fuzzyThreshold}
                        onChange={e => save({ ...local, fuzzyThreshold: Number(e.target.value) })}
                    />
                </div>
                {/* Reload Interval */}
                <div className="form-control w-full">
                    <label className="label"><span className="label-text">Reload Interval (min)</span></label>
                    <input
                        type="number"
                        min={1}
                        className="input input-bordered input-sm w-24"
                        value={local.reloadIntervalMinutes}
                        onChange={e => save({ ...local, reloadIntervalMinutes: Math.max(1, Number(e.target.value)) })}
                    />
                </div>

                {/* Bookmarks */}
                <h2 className="text-xs font-bold uppercase tracking-wider text-base-content/50 mt-4">Bookmarks</h2>

                <div className="form-control w-full">
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

                <div className="form-control w-full">
                    <label className="label"><span className="label-text">Browser</span></label>
                    <select
                        className="select select-bordered select-sm w-32"
                        value={local.bookmarks.browser}
                        onChange={e => save({ ...local, bookmarks: { ...local.bookmarks, browser: e.target.value as typeof BROWSERS[number] } })}
                    >
                        {BROWSERS.map(b => <option key={b} value={b}>{b}</option>)}
                    </select>
                </div>

                {/* Applications */}
                <h2 className="text-xs font-bold uppercase tracking-wider text-base-content/50 mt-4">Applications</h2>

                <div className="form-control w-full">
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

                <div className="form-control w-full">
                    <label className="label">
                        <span className="label-text">Directories</span>
                        <button className="btn btn-ghost btn-xs gap-1" onClick={addDirectory}>
                            <LuPlus size={12} /> Add
                        </button>
                    </label>
                    <ul className="space-y-1">
                        {local.applications.directories.map((dir, i) => (
                            <li key={i} className="flex items-center gap-2 text-xs bg-base-200 rounded px-2 py-1">
                                <span className="flex-1 truncate">{dir}</span>
                                <button className="btn btn-ghost btn-xs p-0" onClick={() => removeDirectory(i)}><LuX size={12} /></button>
                            </li>
                        ))}
                    </ul>
                </div>

                <div className="form-control w-full">
                    <label className="label">
                        <span className="label-text">Extensions</span>
                        <div className="flex gap-1">
                            <input
                                className="input input-xs input-bordered w-20"
                                placeholder="e.g. lnk"
                                value={newExtension}
                                onChange={e => setNewExtension(e.target.value)}
                                onKeyDown={e => { if (e.key === 'Enter') addExtension(); }}
                            />
                            <button className="btn btn-ghost btn-xs" onClick={addExtension}><LuPlus size={12} /></button>
                        </div>
                    </label>
                    <div className="flex flex-wrap gap-1">
                        {local.applications.extensions.map((ext, i) => (
                            <span key={i} className="badge badge-sm gap-1">
                                .{ext}
                                <button onClick={() => removeExtension(i)}><LuX size={10} /></button>
                            </span>
                        ))}
                    </div>
                </div>

                {/* Markdown */}
                <h2 className="text-xs font-bold uppercase tracking-wider text-base-content/50 mt-4">Markdown</h2>

                <div className="form-control w-full">
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

                <div className="form-control w-full">
                    <label className="label">
                        <span className="label-text">Files</span>
                        <button className="btn btn-ghost btn-xs gap-1" onClick={addMarkdownPath}>
                            <LuPlus size={12} /> Add
                        </button>
                    </label>
                    <ul className="space-y-1">
                        {local.markdown.paths.map((p, i) => (
                            <li key={i} className="flex items-center gap-2 text-xs bg-base-200 rounded px-2 py-1">
                                <span className="flex-1 truncate">{p}</span>
                                <button className="btn btn-ghost btn-xs p-0" onClick={() => removeMarkdownPath(i)}><LuX size={12} /></button>
                            </li>
                        ))}
                    </ul>
                </div>

                {/* Ignored */}
                <h2 className="text-xs font-bold uppercase tracking-wider text-base-content/50 mt-4">Ignored</h2>

                <div className="form-control w-full pb-4">
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

            </div>
        </div>
    );
};
