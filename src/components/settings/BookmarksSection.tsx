import React from 'react';
import { BookmarkSettings, BROWSERS, Settings } from '../../types';
import { Row, Section } from '../SettingsLayout';

interface BookmarksSectionProps {
    settings: Settings;
    onChange: (settings: Settings) => void;
}

export const BookmarksSection: React.FC<BookmarksSectionProps> = ({ settings, onChange }) => (
    <Section
        title="Bookmarks"
        enabled={{
            value: settings.bookmarks.enabled,
            onChange: (enabled) =>
                onChange({ ...settings, bookmarks: { ...settings.bookmarks, enabled } }),
        }}
    >
        <Row label="Browser" htmlFor="cfg-browser">
            <select
                id="cfg-browser"
                className="select select-sm w-40"
                value={settings.bookmarks.browser}
                onChange={(e) =>
                    onChange({
                        ...settings,
                        bookmarks: {
                            ...settings.bookmarks,
                            browser: e.target.value as BookmarkSettings['browser'],
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
