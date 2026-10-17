import React from 'react';
import { Config } from '../../types';
import { ListRow, Section } from '../SettingsLayout';

interface IgnoredSectionProps {
    config: Config;
    onChange: (config: Config) => void;
}

export const IgnoredSection: React.FC<IgnoredSectionProps> = ({ config, onChange }) => (
    <Section title="Ignored">
        <ListRow
            label="Hidden from search"
            description="Add from the ⋮ menu on a result."
            items={config.ignored}
            onRemove={(idx) =>
                onChange({ ...config, ignored: config.ignored.filter((_, i) => i !== idx) })
            }
            emptyText="No ignored commands."
        />
    </Section>
);
