import {
    parseArgs,
    expandTemplate,
    hasPlaceholders,
    missingSlots,
    slotsOf,
    substituteArgs,
    unusedArgs,
} from '../command';

describe('hasPlaceholders', () => {
    it('detects {$1}', () => expect(hasPlaceholders('open {$1}')).toBe(true));
    it('detects {$*}', () => expect(hasPlaceholders('search {$*}')).toBe(true));
    it('detects {$99}', () => expect(hasPlaceholders('cmd {$99}')).toBe(true));
    it('returns false for plain string', () => expect(hasPlaceholders('open browser')).toBe(false));
    it('returns false for empty string', () => expect(hasPlaceholders('')).toBe(false));
    it('returns false for {$} without digit or *', () =>
        expect(hasPlaceholders('{$}')).toBe(false));
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

    it('replaces both {$*} and positional placeholders (same as the backend)', () => {
        expect(substituteArgs('{$*} and {$1}', ['a', 'b'])).toBe('a b and a');
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
    it('replaces every occurrence of a placeholder', () => {
        expect(substituteArgs('{$1}-{$1}', ['x'])).toBe('x-x');
    });

    it('does not interpret $ patterns in args', () => {
        expect(substituteArgs('echo {$1}', ['$&'])).toBe('echo $&');
    });

    it('percent-encodes args when urlEncode is set', () => {
        expect(substituteArgs('https://x.com/?q={$*}', ['c++', 'a&b'], true)).toBe(
            'https://x.com/?q=c%2B%2B%20a%26b',
        );
    });
});

describe('expandTemplate', () => {
    it('keeps literal text and marks which placeholders were filled', () => {
        expect(expandTemplate('https://x.dev/?q={$1}&l={$2}', ['react'])).toEqual([
            { text: 'https://x.dev/?q=', filled: false },
            { text: 'react', slot: '{$1}', filled: true },
            { text: '&l=', filled: false },
            { text: '{$2}', slot: '{$2}', filled: false },
        ]);
    });

    it('fills {$*} with every argument and nothing when there are none', () => {
        expect(expandTemplate('go {$*}', ['a', 'b'])[1]).toEqual({
            text: 'a b',
            slot: '{$*}',
            filled: true,
        });
        expect(expandTemplate('go {$*}', [])[1]).toEqual({
            text: '{$*}',
            slot: '{$*}',
            filled: false,
        });
    });

    it('returns a single literal part for a command without placeholders', () => {
        expect(expandTemplate('ls -la', ['x'])).toEqual([{ text: 'ls -la', filled: false }]);
    });
});

describe('placeholder bookkeeping', () => {
    it('lists the distinct placeholders in order', () => {
        expect(slotsOf('{$2} {$1} {$2}')).toEqual(['{$2}', '{$1}']);
    });

    it('reports placeholders still waiting for a value', () => {
        expect(missingSlots('cmd {$1} {$2}', ['a'])).toEqual(['{$2}']);
        expect(missingSlots('cmd {$*}', [])).toEqual(['{$*}']);
        expect(missingSlots('cmd {$1}', ['a'])).toEqual([]);
    });

    it('reports arguments the command would silently drop', () => {
        expect(unusedArgs('https://x.dev/?q={$1}', ['react', 'hooks'])).toEqual(['hooks']);
        expect(unusedArgs('cmd {$1} {$2}', ['a', 'b'])).toEqual([]);
        expect(unusedArgs('no placeholders', ['a'])).toEqual(['a']);
    });

    it('never calls arguments unused when {$*} takes them all', () => {
        expect(unusedArgs('cmd {$*}', ['a', 'b', 'c'])).toEqual([]);
    });
});

describe('parseArgs', () => {
    it('returns the words after the alias', () => {
        expect(parseArgs('d  foo bar ', 'd')).toEqual(['foo', 'bar']);
        expect(parseArgs(' d foo', 'd')).toEqual(['foo']);
    });

    it('returns nothing until an argument is typed', () => {
        expect(parseArgs('d ', 'd')).toEqual([]);
    });

    it('returns nothing for a query that is not for the alias', () => {
        expect(parseArgs('dd foo', 'd')).toEqual([]);
    });
});
