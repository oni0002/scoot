import React from 'react';
import { BookmarkConfig, BROWSERS, Config } from '../../types';
import { Row, Section } from '../SettingsLayout';

interface BookmarksSectionProps {
    config: Config;
    onChange: (config: Config) => void;
}

export const BookmarksSection: React.FC<BookmarksSectionProps> = ({ config, onChange }) => (
    <Section
        title="Bookmarks"
        enabled={{
            value: config.bookmarks.enabled,
            onChange: (enabled) =>
                onChange({ ...config, bookmarks: { ...config.bookmarks, enabled } }),
        }}
    >
        <Row label="Browser" htmlFor="cfg-browser">
            <select
                id="cfg-browser"
                className="select select-sm w-40"
                value={config.bookmarks.browser}
                onChange={(e) =>
                    onChange({
                        ...config,
                        bookmarks: {
                            ...config.bookmarks,
                            browser: e.target.value as BookmarkConfig['browser'],
                        },
                    })
                }
            >
                {BROWSERS.map((b) => (
                    <option key={b} value={b}>
                        {b}
                    </option>
                ))}
            </select>
        </Row>
    </Section>
);
