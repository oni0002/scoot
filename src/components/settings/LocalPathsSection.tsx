import React, { useState } from 'react';
import { LuFolder, LuX } from 'react-icons/lu';
import { TauriAPI } from '../../tauri';
import { useNotificationContext } from '../../context/NotificationContext';
import { LocalPathEntry, Settings } from '../../types';
import { Section, TextListRow } from '../SettingsLayout';
import { NumberInput } from './NumberInput';

interface LocalPathsSectionProps {
    settings: Settings;
    onChange: (settings: Settings) => void;
}

// Rust 側は大文字小文字を区別せずに比べるので、大文字小文字だけ違う値は足さない
const withAdded = (list: string[], values: string[]): string[] =>
    values.reduce(
        (acc, value) =>
            value && !acc.some((v) => v.toLowerCase() === value.toLowerCase())
                ? [...acc, value]
                : acc,
        list,
    );

/** 拡張子の欄。入力中は打ったままの文字を見せ、離れたら保存される形(ドットなし・重複なし)で見せる */
const ExtensionsInput: React.FC<{
    label: string;
    value: string[];
    onChange: (extensions: string[]) => void;
}> = ({ label, value, onChange }) => {
    const [draft, setDraft] = useState<string | null>(null);
    return (
        <input
            type="text"
            autoComplete="off"
            className="input input-xs w-44 placeholder:italic"
            aria-label={label}
            placeholder={draft === null ? 'No files' : 'md, pdf, or * for all files'}
            value={draft ?? value.join(', ')}
            onFocus={() => setDraft(value.join(', '))}
            onChange={(e) => {
                setDraft(e.target.value);
                onChange(
                    withAdded(
                        [],
                        e.target.value.split(/[,\s]+/).map((ext) => ext.replace(/^\./, '')),
                    ),
                );
            }}
            onBlur={() => setDraft(null)}
        />
    );
};

export const LocalPathsSection: React.FC<LocalPathsSectionProps> = ({ settings, onChange }) => {
    const { reportError } = useNotificationContext();
    const [newPath, setNewPath] = useState('');
    const local = settings.localPaths;
    const update = (patch: Partial<Settings['localPaths']>) =>
        onChange({ ...settings, localPaths: { ...local, ...patch } });
    const updateEntry = (index: number, patch: Partial<LocalPathEntry>) =>
        update({ entries: local.entries.map((e, i) => (i === index ? { ...e, ...patch } : e)) });
    // 既定値は src-tauri/src/settings/domain.rs の LocalPathEntry::default と揃える
    const addEntry = (path: string) =>
        update({
            entries: [...local.entries, { path, depth: 3, extensions: [], includeFolders: true }],
        });

    const addTyped = () => {
        const path = newPath.trim();
        if (!path) return;
        addEntry(path);
        setNewPath('');
    };

    const chooseDirectory = async () => {
        try {
            const selected = await TauriAPI.pickPath(true);
            if (selected) addEntry(selected);
        } catch (err) {
            reportError('choose a folder', err);
        }
    };

    return (
        <Section
            title="Files and Folders"
            enabled={{ value: local.enabled, onChange: (enabled) => update({ enabled }) }}
        >
            <div className="px-1.5 py-1.5">
                {/* 狭いウィンドウでは表だけ横にスクロールする。700px の窓では収まる */}
                <div className="overflow-x-auto">
                    <table className="table table-xs">
                        <thead className="text-xs text-subtle">
                            <tr>
                                <th>Path</th>
                                <th>Depth</th>
                                <th>Extensions</th>
                                <th>Folders</th>
                                <th>
                                    <span className="sr-only">Remove</span>
                                </th>
                            </tr>
                        </thead>
                        <tbody>
                            {local.entries.map((entry, i) => (
                                <tr key={i}>
                                    {/* 余りの幅を取り、長いパスは頭を省いて末尾(大事な部分)を見せる */}
                                    <td className="w-full min-w-32 max-w-0">
                                        <span
                                            dir="rtl"
                                            className="block truncate text-left font-mono"
                                            title={entry.path}
                                        >
                                            <span dir="ltr">{entry.path}</span>
                                        </span>
                                    </td>
                                    <td>
                                        {/* max は src-tauri/src/settings/domain.rs の MAX_LOCAL_PATH_DEPTH と揃える */}
                                        <NumberInput
                                            aria-label={`Depth for ${entry.path}`}
                                            className="input-xs w-14"
                                            min={1}
                                            max={10}
                                            value={entry.depth}
                                            onChange={(depth) => updateEntry(i, { depth })}
                                        />
                                    </td>
                                    <td>
                                        <ExtensionsInput
                                            label={`Extensions for ${entry.path}`}
                                            value={entry.extensions}
                                            onChange={(extensions) =>
                                                updateEntry(i, { extensions })
                                            }
                                        />
                                    </td>
                                    <td className="text-center">
                                        <input
                                            type="checkbox"
                                            className="checkbox checkbox-sm"
                                            aria-label={`Folders for ${entry.path}`}
                                            checked={entry.includeFolders}
                                            onChange={(e) =>
                                                updateEntry(i, { includeFolders: e.target.checked })
                                            }
                                        />
                                    </td>
                                    <td>
                                        <button
                                            type="button"
                                            className="btn btn-ghost btn-xs btn-square"
                                            onClick={() =>
                                                update({
                                                    entries: local.entries.filter(
                                                        (_, j) => j !== i,
                                                    ),
                                                })
                                            }
                                            aria-label={`Remove ${entry.path}`}
                                        >
                                            <LuX size={14} />
                                        </button>
                                    </td>
                                </tr>
                            ))}
                        </tbody>
                    </table>
                </div>
                <div className="mt-1 flex items-center gap-1 px-2">
                    <input
                        type="text"
                        autoComplete="off"
                        className="input input-xs min-w-0 flex-1 font-mono"
                        placeholder="Add a folder path"
                        aria-label="New directory"
                        value={newPath}
                        onChange={(e) => setNewPath(e.target.value)}
                        onKeyDown={(e) => {
                            if (e.key === 'Enter') addTyped();
                        }}
                    />
                    <button
                        type="button"
                        className="btn btn-ghost btn-xs btn-square"
                        onClick={chooseDirectory}
                        aria-label="Choose a folder"
                        title="Choose a folder"
                    >
                        <LuFolder size={14} />
                    </button>
                    <button
                        type="button"
                        className="btn btn-primary btn-xs"
                        onClick={addTyped}
                        disabled={!newPath.trim()}
                    >
                        Add directory
                    </button>
                </div>
            </div>
            <TextListRow
                label="Excluded names"
                description="Files and folders skipped with everything inside."
                items={local.excludeNames}
                onRemove={(idx) =>
                    update({ excludeNames: local.excludeNames.filter((_, i) => i !== idx) })
                }
                addLabel="Add name"
                inputLabel="New excluded name"
                placeholder="e.g. node_modules"
                onAdd={(text) => update({ excludeNames: withAdded(local.excludeNames, [text]) })}
            />
        </Section>
    );
};
