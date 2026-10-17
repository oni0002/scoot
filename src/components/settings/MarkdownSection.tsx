import React from 'react';
import { open as openDialog } from '@tauri-apps/plugin-dialog';
import { Config } from '../../types';
import { ListRow, Section } from '../SettingsLayout';

interface MarkdownSectionProps {
    config: Config;
    onChange: (config: Config) => void;
}

export const MarkdownSection: React.FC<MarkdownSectionProps> = ({ config, onChange }) => {
    const md = config.markdown;
    const update = (patch: Partial<Config['markdown']>) =>
        onChange({ ...config, markdown: { ...md, ...patch } });

    const addFile = async () => {
        const selected = await openDialog({ directory: false, multiple: false });
        if (typeof selected === 'string') update({ paths: [...md.paths, selected] });
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
