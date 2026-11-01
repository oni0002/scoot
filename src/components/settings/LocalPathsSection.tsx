import React from 'react';
import { TauriAPI } from '../../tauri';
import { useNotificationContext } from '../../context/NotificationContext';
import { Settings } from '../../types';
import { ListRow, Row, Section, TextListRow } from '../SettingsLayout';
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

export const LocalPathsSection: React.FC<LocalPathsSectionProps> = ({ settings, onChange }) => {
    const { reportError } = useNotificationContext();
    const local = settings.localPaths;
    const update = (patch: Partial<Settings['localPaths']>) =>
        onChange({ ...settings, localPaths: { ...local, ...patch } });

    const addDirectory = async () => {
        try {
            const selected = await TauriAPI.pickPath(true);
            if (selected) update({ paths: [...local.paths, selected] });
        } catch (err) {
            reportError('choose a folder', err);
        }
    };

    return (
        <Section
            title="Files and Folders"
            enabled={{ value: local.enabled, onChange: (enabled) => update({ enabled }) }}
        >
            <ListRow
                label="Directories"
                description="Folders scanned for files and folders."
                items={local.paths}
                onRemove={(idx) => update({ paths: local.paths.filter((_, i) => i !== idx) })}
                addLabel="Add directory"
                onAdd={addDirectory}
            />
            <Row
                label="Depth"
                htmlFor="cfg-local-depth"
                description="How many folder levels to scan."
            >
                {/* max は src-tauri/src/settings/domain.rs の MAX_LOCAL_PATH_DEPTH と揃える */}
                <NumberInput
                    id="cfg-local-depth"
                    min={1}
                    max={10}
                    value={local.depth}
                    onChange={(depth) => update({ depth })}
                />
            </Row>
            <TextListRow
                label="Extensions"
                description="File types to list."
                items={local.extensions.map((ext) => (ext === '*' ? ext : `.${ext}`))}
                emptyText="No files"
                onRemove={(idx) =>
                    update({ extensions: local.extensions.filter((_, i) => i !== idx) })
                }
                addLabel="Add extension"
                inputLabel="New extension"
                placeholder="md, pdf, or * for all files"
                onAdd={(text) =>
                    update({
                        extensions: withAdded(
                            local.extensions,
                            text.split(/[,\s]+/).map((ext) => ext.replace(/^\./, '')),
                        ),
                    })
                }
            />
            <Row
                label="Include folders"
                htmlFor="cfg-local-folders"
                description="List folders as well as files."
            >
                <input
                    id="cfg-local-folders"
                    type="checkbox"
                    className="toggle"
                    checked={local.includeFolders}
                    onChange={(e) => update({ includeFolders: e.target.checked })}
                />
            </Row>
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
