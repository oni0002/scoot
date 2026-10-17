import React, { useState } from 'react';
import { open as openDialog } from '@tauri-apps/plugin-dialog';
import { Config } from '../../types';
import { ListRow, Section } from '../SettingsLayout';

interface ApplicationsSectionProps {
    config: Config;
    onChange: (config: Config) => void;
}

export const ApplicationsSection: React.FC<ApplicationsSectionProps> = ({ config, onChange }) => {
    const apps = config.applications;
    const update = (patch: Partial<Config['applications']>) =>
        onChange({ ...config, applications: { ...apps, ...patch } });

    const [newExtension, setNewExtension] = useState('');
    const [addingExtension, setAddingExtension] = useState(false);

    const addDirectory = async () => {
        const selected = await openDialog({ directory: true, multiple: false });
        if (typeof selected === 'string') update({ directories: [...apps.directories, selected] });
    };

    const closeExtensionEditor = () => {
        setAddingExtension(false);
        setNewExtension('');
    };

    const confirmAddExtension = () => {
        const ext = newExtension.trim().replace(/^\./, '');
        if (ext && !apps.extensions.includes(ext))
            update({ extensions: [...apps.extensions, ext] });
        closeExtensionEditor();
    };

    return (
        <Section
            title="Applications"
            enabled={{ value: apps.enabled, onChange: (enabled) => update({ enabled }) }}
        >
            <ListRow
                label="Directories"
                description="Folders scanned for application shortcuts."
                items={apps.directories}
                onRemove={(idx) =>
                    update({ directories: apps.directories.filter((_, i) => i !== idx) })
                }
                addLabel="Add directory"
                onAdd={addDirectory}
            />
            <ListRow
                label="Extensions"
                description="File types treated as applications."
                items={apps.extensions.map((ext) => `.${ext}`)}
                onRemove={(idx) =>
                    update({ extensions: apps.extensions.filter((_, i) => i !== idx) })
                }
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
                                    if (e.key === 'Escape') closeExtensionEditor();
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
                                onClick={closeExtensionEditor}
                            >
                                Cancel
                            </button>
                        </li>
                    )
                }
            />
        </Section>
    );
};
