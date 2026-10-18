import React from 'react';
import { Settings } from '../../types';
import { ListRow, Section } from '../SettingsLayout';

interface IgnoredSectionProps {
    settings: Settings;
    onChange: (settings: Settings) => void;
}

export const IgnoredSection: React.FC<IgnoredSectionProps> = ({ settings, onChange }) => (
    <Section title="Ignored">
        <ListRow
            label="Hidden from search"
            description="Add from the ⋮ menu on a result."
            items={settings.ignored}
            onRemove={(idx) =>
                onChange({ ...settings, ignored: settings.ignored.filter((_, i) => i !== idx) })
            }
            emptyText="No ignored commands."
        />
    </Section>
);
