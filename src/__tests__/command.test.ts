import { hasPlaceholders, substituteArgs } from '../command';

describe('hasPlaceholders', () => {
    it('detects {$1}', () => expect(hasPlaceholders('open {$1}')).toBe(true));
    it('detects {$*}', () => expect(hasPlaceholders('search {$*}')).toBe(true));
    it('detects {$99}', () => expect(hasPlaceholders('cmd {$99}')).toBe(true));
    it('returns false for plain string', () => expect(hasPlaceholders('open browser')).toBe(false));
    it('returns false for empty string', () => expect(hasPlaceholders('')).toBe(false));
    it('returns false for {$} without digit or *', () => expect(hasPlaceholders('{$}')).toBe(false));
});

describe('substituteArgs', () => {
    it('returns cmd unchanged when no args', () => {
        expect(substituteArgs('open {$1}', [])).toBe('open {$1}');
    });

    it('replaces {$1} with first arg', () => {
        expect(substituteArgs('open {$1}', ['foo'])).toBe('open foo');
    });

    it('replaces multiple positional args', () => {
        expect(substituteArgs('{$1} to {$2}', ['hello', 'world'])).toBe('hello to world');
    });

    it('replaces {$*} with all args joined by space', () => {
        expect(substituteArgs('search {$*}', ['a', 'b', 'c'])).toBe('search a b c');
    });

    it('{$*} takes priority: {$1} remains when {$*} is present', () => {
        // {$*} branch exits early — {$1} is not replaced
        expect(substituteArgs('{$*} and {$1}', ['a', 'b'])).toBe('a b and {$1}');
    });

    it('leaves unreplaced placeholder when args are insufficient (regression)', () => {
        // {$2} remains if only one arg provided — current behavior, not a bug
        expect(substituteArgs('cmd {$1} {$2}', ['only'])).toBe('cmd only {$2}');
    });

    it('leaves cmd unchanged when no placeholders and args provided', () => {
        expect(substituteArgs('open browser', ['ignored'])).toBe('open browser');
    });

    it('handles arg containing spaces in {$1}', () => {
        expect(substituteArgs('echo {$1}', ['hello world'])).toBe('echo hello world');
    });
});
