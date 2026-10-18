import React, { useState } from 'react';
import { TauriAPI } from '../../tauri';
import { useNotificationContext } from '../../context/NotificationContext';
import { Settings } from '../../types';
import { ListRow, Section } from '../SettingsLayout';

interface ApplicationsSectionProps {
    settings: Settings;
    onChange: (settings: Settings) => void;
}

export const ApplicationsSection: React.FC<ApplicationsSectionProps> = ({ settings, onChange }) => {
    const apps = settings.applications;
    const update = (patch: Partial<Settings['applications']>) =>
        onChange({ ...settings, applications: { ...apps, ...patch } });

    const { reportError } = useNotificationContext();
    const [newExtension, setNewExtension] = useState('');
    const [addingExtension, setAddingExtension] = useState(false);

    const addDirectory = async () => {
        try {
            const selected = await TauriAPI.pickPath(true);
            if (selected) update({ directories: [...apps.directories, selected] });
        } catch (err) {
            reportError('choose a folder', err);
        }
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
                                    if (e.key === 'Escape') {
                                        // Only this field is closed, not the screen around it
                                        e.preventDefault();
                                        closeExtensionEditor();
                                    }
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
