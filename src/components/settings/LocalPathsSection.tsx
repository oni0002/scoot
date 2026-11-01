import React, { useRef, useState } from 'react';
import { LuCheck, LuFolder, LuPencil, LuX } from 'react-icons/lu';
import { TauriAPI } from '../../tauri';
import { useNotificationContext } from '../../context/NotificationContext';
import { LocalPathEntry, Settings } from '../../types';
import { ADD_BUTTON, LIST_SURFACE, Section, TextListRow } from '../SettingsLayout';
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
    const local = settings.localPaths;
    const update = (patch: Partial<Settings['localPaths']>) =>
        onChange({ ...settings, localPaths: { ...local, ...patch } });
    const updateEntry = (index: number, patch: Partial<LocalPathEntry>) =>
        update({ entries: local.entries.map((e, i) => (i === index ? { ...e, ...patch } : e)) });
    // 編集中の行の位置。行を消したら、この位置も一緒にずらす
    const [editing, setEditing] = useState<number | null>(null);
    // 足したばかりの行の位置。パス欄が開いたときに一度だけフォーカスする(Edit ボタンのフォーカスは奪わない)
    const focusPath = useRef<number | null>(null);
    const removeEntry = (index: number) => {
        update({ entries: local.entries.filter((_, i) => i !== index) });
        setEditing((e) => (e === null || e === index ? null : e > index ? e - 1 : e));
    };
    // 編集する行を切り替える。離れる行のパスは前後の空白を落とし、空なら黙って捨てる(保存時の検証で弾かれるため)
    const switchEdit = (next: number | null) => {
        let target = next;
        if (editing !== null && editing !== next) {
            const path = local.entries[editing].path.trim();
            if (path) {
                updateEntry(editing, { path });
            } else {
                update({ entries: local.entries.filter((_, i) => i !== editing) });
                if (target !== null && target > editing) target -= 1;
            }
        }
        setEditing(target);
    };
    // 既定値は src-tauri/src/settings/domain.rs の LocalPathEntry::default と揃える。足した行は編集で開く
    const addEntry = () => {
        update({
            entries: [
                ...local.entries,
                { path: '', depth: 3, extensions: [], includeFolders: true },
            ],
        });
        focusPath.current = local.entries.length;
        setEditing(local.entries.length);
    };

    const chooseDirectory = async (index: number) => {
        try {
            const selected = await TauriAPI.pickPath(true);
            if (selected) updateEntry(index, { path: selected });
        } catch (err) {
            reportError('choose a folder', err);
        }
    };

    return (
        <Section
            title="Files and Folders"
            enabled={{ value: local.enabled, onChange: (enabled) => update({ enabled }) }}
        >
            <div className="px-3 py-2.5">
                {/* 狭いウィンドウでは表だけ横にスクロールする。列幅は固定で、表示と編集で動かさない */}
                <div className={`overflow-x-auto ${LIST_SURFACE}`}>
                    <table className="table table-xs min-w-[34rem] table-fixed">
                        <colgroup>
                            <col />
                            <col className="w-20" />
                            <col className="w-48" />
                            <col className="w-20" />
                            <col className="w-20" />
                        </colgroup>
                        <thead className="text-xs text-subtle">
                            <tr className="border-base-content/10">
                                <th>Path</th>
                                <th>Depth</th>
                                <th>Extensions</th>
                                <th>Folders</th>
                                <th>
                                    <span className="sr-only">Actions</span>
                                </th>
                            </tr>
                        </thead>
                        <tbody>
                            {local.entries.map((entry, i) => {
                                const isEditing = editing === i;
                                const name = entry.path || 'new path';
                                return (
                                    <tr key={i} className="border-base-content/10">
                                        <td>
                                            {isEditing ? (
                                                <div className="flex items-center gap-1">
                                                    <input
                                                        type="text"
                                                        autoComplete="off"
                                                        className="input input-xs min-w-0 flex-1 font-mono"
                                                        aria-label="Path"
                                                        placeholder="Add a folder path"
                                                        value={entry.path}
                                                        ref={(el) => {
                                                            if (el && focusPath.current === i) {
                                                                focusPath.current = null;
                                                                el.focus();
                                                            }
                                                        }}
                                                        onChange={(e) =>
                                                            updateEntry(i, { path: e.target.value })
                                                        }
                                                    />
                                                    <button
                                                        type="button"
                                                        className="btn btn-ghost btn-xs btn-square flex-none"
                                                        onClick={() => chooseDirectory(i)}
                                                        aria-label="Choose a folder"
                                                        title="Choose a folder"
                                                    >
                                                        <LuFolder size={14} />
                                                    </button>
                                                </div>
                                            ) : (
                                                // 長いパスは頭を省いて末尾(大事な部分)を見せる
                                                <span
                                                    dir="rtl"
                                                    className="block truncate text-left font-mono"
                                                    title={entry.path}
                                                >
                                                    <span dir="ltr">{entry.path}</span>
                                                </span>
                                            )}
                                        </td>
                                        <td>
                                            {isEditing ? (
                                                // max は src-tauri/src/settings/domain.rs の MAX_LOCAL_PATH_DEPTH と揃える
                                                <NumberInput
                                                    aria-label={`Depth for ${name}`}
                                                    className="input-xs w-14"
                                                    min={1}
                                                    max={10}
                                                    value={entry.depth}
                                                    onChange={(depth) => updateEntry(i, { depth })}
                                                />
                                            ) : (
                                                entry.depth
                                            )}
                                        </td>
                                        <td>
                                            {isEditing ? (
                                                <ExtensionsInput
                                                    label={`Extensions for ${name}`}
                                                    value={entry.extensions}
                                                    onChange={(extensions) =>
                                                        updateEntry(i, { extensions })
                                                    }
                                                />
                                            ) : entry.extensions.length > 0 ? (
                                                <span className="block truncate">
                                                    {entry.extensions.join(', ')}
                                                </span>
                                            ) : (
                                                <span className="italic text-subtle">No files</span>
                                            )}
                                        </td>
                                        <td>
                                            {isEditing ? (
                                                <input
                                                    type="checkbox"
                                                    className="checkbox checkbox-sm"
                                                    aria-label={`Folders for ${name}`}
                                                    checked={entry.includeFolders}
                                                    onChange={(e) =>
                                                        updateEntry(i, {
                                                            includeFolders: e.target.checked,
                                                        })
                                                    }
                                                />
                                            ) : entry.includeFolders ? (
                                                'Yes'
                                            ) : (
                                                'No'
                                            )}
                                        </td>
                                        <td>
                                            <div className="flex justify-end gap-0.5">
                                                {/* 編集と完了は同じボタンにして、押した後もフォーカスを保つ */}
                                                <button
                                                    type="button"
                                                    className="btn btn-ghost btn-xs btn-square"
                                                    onClick={() => switchEdit(isEditing ? null : i)}
                                                    aria-label={`${isEditing ? 'Done editing' : 'Edit'} ${name}`}
                                                    title={isEditing ? 'Done' : 'Edit'}
                                                >
                                                    {isEditing ? (
                                                        <LuCheck size={14} />
                                                    ) : (
                                                        <LuPencil size={14} />
                                                    )}
                                                </button>
                                                <button
                                                    type="button"
                                                    className="btn btn-ghost btn-xs btn-square"
                                                    onClick={() => removeEntry(i)}
                                                    aria-label={`Remove ${name}`}
                                                    title="Remove"
                                                >
                                                    <LuX size={14} />
                                                </button>
                                            </div>
                                        </td>
                                    </tr>
                                );
                            })}
                        </tbody>
                    </table>
                </div>
                {/* 空のパスの行が開いている間は、もう1行足さない */}
                <button
                    type="button"
                    className={ADD_BUTTON}
                    onClick={addEntry}
                    disabled={local.entries.some((e) => !e.path.trim())}
                >
                    Add path
                </button>
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
