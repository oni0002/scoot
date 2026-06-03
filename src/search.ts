import Fuse, { IFuseOptions } from 'fuse.js';
import { Command, SearchResult } from './types';
import { DIRECT_OPEN_ID } from './constants';

export const FUSE_KEYS: IFuseOptions<Command>['keys'] = [
    { name: 'name', weight: 0.7 },
    { name: 'alias', weight: 0.5 },
];

const DEFAULT_FUSE_OPTIONS: IFuseOptions<Command> = {
    keys: FUSE_KEYS,
    includeScore: true,
    includeMatches: true,
    minMatchCharLength: 1,
    ignoreLocation: false,
    findAllMatches: true,
    useExtendedSearch: false,
};

export function createFuse(commands: Command[], threshold: number): Fuse<Command> {
    const clamped = Math.max(0, Math.min(1, threshold));
    const options = { ...DEFAULT_FUSE_OPTIONS, threshold: clamped };
    const index = Fuse.createIndex(FUSE_KEYS!, commands);
    return new Fuse(commands, options, index);
}

export function fuseSearch(
    fuse: Fuse<Command>,
    query: string,
    commands: Command[],
    maxResults: number,
): SearchResult[] {
    if (!query.trim()) {
        return commands.slice(0, maxResults).map((command) => ({ command, score: 0, matches: [] }));
    }
    return fuse.search(query, { limit: maxResults }).map((r) => ({
        command: r.item,
        score: r.score ?? 0,
        matches: r.matches?.map((m) => ({ indices: m.indices, key: m.key ?? '' })) ?? [],
    }));
}

const URL_PATTERN = /^(https?|ftp|file|mailto|scoot):\/\/?/i;
const UNC_PATH_PATTERN = /^\\\\/i;
const WIN_ABS_PATH_PATTERN = /^[A-Za-z]:\\/i;
const UNIX_ABS_PATH_PATTERN = /^(\/|~\/)/i;

export function detectDirectOpen(query: string): Command | null {
    const trimmed = query.trim();
    if (!trimmed) return null;
    const isUrl = URL_PATTERN.test(trimmed);
    const isPath =
        WIN_ABS_PATH_PATTERN.test(trimmed) ||
        UNC_PATH_PATTERN.test(trimmed) ||
        UNIX_ABS_PATH_PATTERN.test(trimmed);
    if (!isUrl && !isPath) return null;
    return {
        id: DIRECT_OPEN_ID,
        name: `Open: ${trimmed}`,
        category: isUrl ? 'url' : 'file',
        source: 'user',
        command: trimmed,
        description: `Open ${isUrl ? 'url' : 'file'}`,
    };
}
