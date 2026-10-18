import React from 'react';
import { TauriAPI } from '../../tauri';
import { useNotificationContext } from '../../context/NotificationContext';
import { Config } from '../../types';
import { ListRow, Section } from '../SettingsLayout';

interface MarkdownSectionProps {
    config: Config;
    onChange: (config: Config) => void;
}

export const MarkdownSection: React.FC<MarkdownSectionProps> = ({ config, onChange }) => {
    const { reportError } = useNotificationContext();
    const md = config.markdown;
    const update = (patch: Partial<Config['markdown']>) =>
        onChange({ ...config, markdown: { ...md, ...patch } });

    const addFile = async () => {
        try {
            const selected = await TauriAPI.pickPath(false);
            if (selected) update({ paths: [...md.paths, selected] });
        } catch (err) {
            reportError('choose a file', err);
        }
    };

    return (
        <Section
            title="Markdown"
            enabled={{ value: md.enabled, onChange: (enabled) => update({ enabled }) }}
        >
            <ListRow
                label="Files"
                description="Markdown files to read links from."
                items={md.paths}
                onRemove={(idx) => update({ paths: md.paths.filter((_, i) => i !== idx) })}
                addLabel="Add file"
                onAdd={addFile}
            />
        </Section>
    );
};
