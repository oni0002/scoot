import Fuse, { IFuseOptions } from 'fuse.js';
import { Command, SearchResult, Usage, UsageMap } from './types';
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

// Run history nudges the order of similar matches; it must never lift a poor match over a good one.
// Fuse scores run from 0 (exact) to 1, so the boost is capped well below the gap between a good
// and a mediocre match.
//
//   weight = count * 0.5 ^ (daysSinceLastUse / USAGE_HALF_LIFE_DAYS)
//   boost  = MAX_USAGE_BOOST * weight / (weight + USAGE_SATURATION)
//   rank   = fuseScore - boost            (lower ranks first)
//
// The boost rises with the weight but saturates at MAX_USAGE_BOOST, so a command run a thousand
// times cannot bury a better match. Decay uses only the last run, not each run's own age.
const MAX_USAGE_BOOST = 0.05;
const USAGE_HALF_LIFE_DAYS = 30;
const USAGE_SATURATION = 3;
const DAY_MS = 24 * 60 * 60 * 1000;

/** Frecency: each run counts, but runs fade with the time since the last one. */
export function usageBoost(usage: Usage | undefined, now: number): number {
    if (!usage) return 0;
    const ageDays = Math.max(0, now - usage.lastUsed) / DAY_MS;
    const weight = usage.count * Math.pow(0.5, ageDays / USAGE_HALF_LIFE_DAYS);
    return (MAX_USAGE_BOOST * weight) / (weight + USAGE_SATURATION);
}

export function fuseSearch(
    fuse: Fuse<Command>,
    query: string,
    commands: Command[],
    maxResults: number,
    usage: UsageMap = {},
    now: number = Date.now(),
): SearchResult[] {
    if (!query.trim()) {
        return commands.slice(0, maxResults).map((command) => ({ command, score: 0, matches: [] }));
    }
    // Scattered one-letter hits are kept and highlighted like longer runs, so a result
    // never looks unexplained.
    // Re-ranking by history needs every match, not just the first maxResults
    return fuse
        .search(query)
        .map((r) => ({
            rank: (r.score ?? 0) - usageBoost(usage[r.item.id], now),
            result: {
                command: r.item,
                score: r.score ?? 0,
                matches:
                    r.matches?.map((m) => ({ indices: [...m.indices], key: m.key ?? '' })) ?? [],
            },
        }))
        .sort((a, b) => a.rank - b.rank)
        .slice(0, maxResults)
        .map((r) => r.result);
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
