import React from 'react';
import { TauriAPI } from '../../tauri';
import { useNotificationContext } from '../../context/NotificationContext';
import { Settings } from '../../types';
import { ListRow, Section, TextListRow } from '../SettingsLayout';

interface ApplicationsSectionProps {
    settings: Settings;
    onChange: (settings: Settings) => void;
}

export const ApplicationsSection: React.FC<ApplicationsSectionProps> = ({ settings, onChange }) => {
    const apps = settings.applications;
    const update = (patch: Partial<Settings['applications']>) =>
        onChange({ ...settings, applications: { ...apps, ...patch } });

    const { reportError } = useNotificationContext();

    const addDirectory = async () => {
        try {
            const selected = await TauriAPI.pickPath(true);
            if (selected) update({ directories: [...apps.directories, selected] });
        } catch (err) {
            reportError('choose a folder', err);
        }
    };

    const addExtension = (text: string) => {
        const ext = text.replace(/^\./, '');
        if (ext && !apps.extensions.includes(ext))
            update({ extensions: [...apps.extensions, ext] });
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
            <TextListRow
                label="Extensions"
                description="File types treated as applications."
                items={apps.extensions.map((ext) => `.${ext}`)}
                onRemove={(idx) =>
                    update({ extensions: apps.extensions.filter((_, i) => i !== idx) })
                }
                addLabel="Add extension"
                inputLabel="New extension"
                placeholder="e.g. lnk"
                onAdd={addExtension}
            />
        </Section>
    );
};
