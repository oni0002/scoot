import { detectDirectOpen, createFuse, fuseSearch, usageBoost } from '../search';
import { DIRECT_OPEN_ID } from '../constants';
import { Command } from '../types';

describe('detectDirectOpen', () => {
    it('returns null for empty string', () => expect(detectDirectOpen('')).toBeNull());
    it('returns null for whitespace only', () => expect(detectDirectOpen('   ')).toBeNull());
    it('returns null for plain keyword', () => expect(detectDirectOpen('google')).toBeNull());
    it('returns null for bare domain', () => expect(detectDirectOpen('google.com')).toBeNull());

    it('detects https URL', () => {
        const result = detectDirectOpen('https://example.com');
        expect(result).not.toBeNull();
        expect(result!.id).toBe(DIRECT_OPEN_ID);
        expect(result!.category).toBe('url');
    });

    it('detects http URL', () => {
        expect(detectDirectOpen('http://example.com')?.category).toBe('url');
    });

    it('detects ftp URL', () => {
        expect(detectDirectOpen('ftp://files.example.com')?.category).toBe('url');
    });

    it('detects file: URL as url category', () => {
        expect(detectDirectOpen('file:///C:/foo')?.category).toBe('url');
    });

    it('detects scoot:// as url category', () => {
        expect(detectDirectOpen('scoot://foo')?.category).toBe('url');
    });

    it('detects Windows absolute path', () => {
        const result = detectDirectOpen('C:\\Users\\foo\\bar.txt');
        expect(result!.category).toBe('file');
    });

    it('detects UNC path', () => {
        expect(detectDirectOpen('\\\\server\\share')?.category).toBe('file');
    });

    it('detects Unix absolute path', () => {
        expect(detectDirectOpen('/usr/bin/bash')?.category).toBe('file');
    });

    it('detects home-relative path', () => {
        expect(detectDirectOpen('~/documents')?.category).toBe('file');
    });

    it('trims whitespace before matching', () => {
        expect(detectDirectOpen('  https://example.com  ')?.category).toBe('url');
    });

    it('sets command to trimmed query', () => {
        const result = detectDirectOpen('  https://example.com  ');
        expect(result!.command).toBe('https://example.com');
    });
});

const makeCommand = (name: string, id = name): Command => ({
    id,
    name,
    category: 'url',
    source: 'user',
    command: 'https://example.com',
    description: '',
});

describe('fuseSearch', () => {
    const commands = [
        makeCommand('GitHub'),
        makeCommand('Google'),
        makeCommand('Gmail'),
        makeCommand('Visual Studio Code'),
    ];
    const fuse = createFuse(commands, 0.4);

    it('returns all commands (up to maxResults) for empty query', () => {
        const results = fuseSearch(fuse, '', commands, 10);
        expect(results).toHaveLength(4);
        expect(results[0].score).toBe(0);
    });

    it('respects maxResults for empty query', () => {
        const results = fuseSearch(fuse, '', commands, 2);
        expect(results).toHaveLength(2);
    });

    it('returns matching results for a query', () => {
        const results = fuseSearch(fuse, 'git', commands, 10);
        expect(results.some((r) => r.command.name === 'GitHub')).toBe(true);
    });

    it('returns empty array for query with no matches', () => {
        const results = fuseSearch(fuse, 'xyzzy', commands, 10);
        expect(results).toHaveLength(0);
    });

    it('trims whitespace-only query and returns all commands', () => {
        const results = fuseSearch(fuse, '   ', commands, 10);
        expect(results).toHaveLength(4);
    });
});

describe('fuseSearch highlights', () => {
    const cmd = (name: string): Command => ({
        id: name,
        name,
        category: 'url',
        source: 'user',
        command: 'https://example.com',
        description: '',
    });

    it('keeps scattered one-letter hits so every result shows why it matched', () => {
        const commands = [cmd('Discord Inc')];
        const fuse = createFuse(commands, 0.6);
        const [hit] = fuseSearch(fuse, 'dc', commands, 10);
        const runs = hit.matches.flatMap((m) => m.indices);
        expect(runs.length).toBeGreaterThan(0);
    });

    it('still highlights when the query itself is one letter', () => {
        const commands = [cmd('GitHub')];
        const fuse = createFuse(commands, 0.5);
        const [hit] = fuseSearch(fuse, 'g', commands, 10);
        expect(hit.matches.flatMap((m) => m.indices).length).toBeGreaterThan(0);
    });
});

describe('run history', () => {
    const NOW = Date.UTC(2026, 9, 18);
    const DAY = 24 * 60 * 60 * 1000;
    const commands = [makeCommand('Docs A'), makeCommand('Docs B'), makeCommand('Zebra')];
    const fuse = createFuse(commands, 0.6);
    const names = (usage: Record<string, { count: number; lastUsed: number }>) =>
        fuseSearch(fuse, 'docs', commands, 10, usage, NOW).map((r) => r.command.name);

    it('keeps the match order when nothing was run', () => {
        expect(names({})).toEqual(['Docs A', 'Docs B']);
    });

    it('puts the command run more often first among equal matches', () => {
        expect(names({ 'Docs B': { count: 5, lastUsed: NOW } })[0]).toBe('Docs B');
    });

    it('lets a recent run outweigh an old one', () => {
        const usage = {
            'Docs A': { count: 5, lastUsed: NOW - 365 * DAY },
            'Docs B': { count: 2, lastUsed: NOW },
        };
        expect(names(usage)[0]).toBe('Docs B');
    });

    it('never lifts a poor match over a clearly better one', () => {
        const list = [makeCommand('Terminal'), makeCommand('Term sheet notes archive')];
        const f = createFuse(list, 0.6);
        const usage = { 'Term sheet notes archive': { count: 1000, lastUsed: NOW } };
        const [first] = fuseSearch(f, 'terminal', list, 10, usage, NOW);
        expect(first.command.name).toBe('Terminal');
    });

    it('caps the boost', () => {
        expect(usageBoost({ count: 1e6, lastUsed: NOW }, NOW)).toBeLessThanOrEqual(0.05);
        expect(usageBoost(undefined, NOW)).toBe(0);
    });
});

describe('abbreviations', () => {
    const names = [
        'Comet',
        'Node.js',
        'Visual Studio Code',
        'RecoveryDrive',
        'oni0002/vscode-todomd',
    ];
    const commands = names.map((n) => makeCommand(n));
    const fuse = createFuse(commands, 0.5);

    it('ranks word-start abbreviations above scattered Fuse hits', () => {
        const results = fuseSearch(fuse, 'vscode', commands, 10);
        const order = results.map((r) => r.command.name);
        expect(order.indexOf('Visual Studio Code')).toBeLessThan(order.indexOf('Comet'));
        expect(order.indexOf('Visual Studio Code')).toBeLessThan(order.indexOf('RecoveryDrive'));
    });

    it('finds a command Fuse alone would drop at a strict threshold', () => {
        const strict = createFuse(commands, 0.3);
        const results = fuseSearch(strict, 'vscode', commands, 10);
        expect(results.map((r) => r.command.name)).toContain('Visual Studio Code');
    });

    it('highlights the matched word starts', () => {
        const [hit] = fuseSearch(fuse, 'vscode', [commands[2]], 10);
        expect(hit.matches[0]).toEqual({
            key: 'name',
            indices: [
                [0, 0],
                [7, 7],
                [14, 17],
            ],
        });
    });

    it('does not duplicate a command that both match', () => {
        const results = fuseSearch(fuse, 'vscode', commands, 10);
        const ids = results.map((r) => r.command.id);
        expect(new Set(ids).size).toBe(ids.length);
    });
});
