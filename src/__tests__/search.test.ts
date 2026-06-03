import { detectDirectOpen, createFuse, fuseSearch } from '../search';
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
        expect(results.some(r => r.command.name === 'GitHub')).toBe(true);
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
