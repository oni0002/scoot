/** Values stored in config.json. Keep in sync with `THEMES` in src-tauri/src/config/domain.rs. */
export const THEME_OPTIONS = [
    { value: 'system', label: 'System' },
    { value: 'light', label: 'Light' },
    { value: 'dark', label: 'Dark' },
] as const;

const DAISY_THEME: Record<string, string> = { light: 'scoot-light', dark: 'scoot-dark' };

/** The `data-theme` to set for a config value. `system` sets none, so the OS light/dark setting decides. */
export const daisyTheme = (theme: string): string | undefined => DAISY_THEME[theme];
