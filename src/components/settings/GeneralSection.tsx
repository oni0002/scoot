import React from 'react';
import { Config } from '../../types';
import { THEME_OPTIONS } from '../../theme';
import { HotkeyCapture } from '../../hooks/useHotkeyCapture';
import { Row, Section } from '../SettingsLayout';

interface GeneralSectionProps {
    config: Config;
    onChange: (config: Config) => void;
    hotkey: HotkeyCapture;
}

export const GeneralSection: React.FC<GeneralSectionProps> = ({ config, onChange, hotkey }) => (
    <Section title="General">
        <Row label="Theme" htmlFor="cfg-theme">
            <select
                id="cfg-theme"
                className="select select-sm w-40"
                value={config.theme}
                onChange={(e) => onChange({ ...config, theme: e.target.value })}
            >
                {THEME_OPTIONS.map((t) => (
                    <option key={t.value} value={t.value}>
                        {t.label}
                    </option>
                ))}
            </select>
        </Row>
        <Row
            label="Hotkey"
            htmlFor="cfg-hotkey"
            description="Click, then press a key combination. Esc cancels."
        >
            <input
                id="cfg-hotkey"
                className="input input-sm w-40 text-center cursor-pointer"
                value={hotkey.capturing ? 'Press a key...' : config.hotkey}
                readOnly
                {...hotkey.inputProps}
            />
        </Row>
        <Row label="Max results" htmlFor="cfg-max-results" description="How many results to show.">
            <input
                id="cfg-max-results"
                type="number"
                min={1}
                max={100}
                className="input input-sm w-24"
                value={config.maxResults}
                onChange={(e) =>
                    onChange({
                        ...config,
                        maxResults: Math.max(1, Math.min(100, Number(e.target.value))),
                    })
                }
            />
        </Row>
        <Row
            label="Fuzzy threshold"
            htmlFor="cfg-fuzzy"
            description="Higher is stricter, lower is more permissive."
        >
            <div className="flex items-center gap-3">
                <input
                    id="cfg-fuzzy"
                    type="range"
                    min={0}
                    max={1}
                    step={0.1}
                    className="range w-44"
                    value={config.fuzzyThreshold}
                    onChange={(e) =>
                        onChange({ ...config, fuzzyThreshold: Number(e.target.value) })
                    }
                />
                <span className="w-6 text-right text-sm tabular-nums">
                    {config.fuzzyThreshold.toFixed(1)}
                </span>
            </div>
        </Row>
        <Row
            label="Reload interval"
            htmlFor="cfg-reload"
            description="Minutes between source reloads."
        >
            <input
                id="cfg-reload"
                type="number"
                min={1}
                className="input input-sm w-24"
                value={config.reloadIntervalMinutes}
                onChange={(e) =>
                    onChange({
                        ...config,
                        reloadIntervalMinutes: Math.max(1, Number(e.target.value)),
                    })
                }
            />
        </Row>
    </Section>
);
