import React from 'react';
import { Config } from '../../types';
import { Row, Section } from '../SettingsLayout';

const BROWSERS = ['brave', 'chrome', 'edge'] as const; // the browsers bookmark.rs can read

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
                            browser: e.target.value as (typeof BROWSERS)[number],
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
