import { THEME_OPTIONS, daisyTheme } from '../theme';

describe('daisyTheme', () => {
    it('maps light and dark to the Scoot themes', () => {
        expect(daisyTheme('light')).toBe('scoot-light');
        expect(daisyTheme('dark')).toBe('scoot-dark');
    });

    it('sets no theme for system, so the OS setting decides', () => {
        expect(daisyTheme('system')).toBeUndefined();
        expect(daisyTheme('')).toBeUndefined();
    });

    it('offers system, light and dark', () => {
        expect(THEME_OPTIONS.map((t) => t.value)).toEqual(['system', 'light', 'dark']);
    });
});
