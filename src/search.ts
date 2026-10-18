import Fuse, { IFuseOptions } from 'fuse.js';
import { Command, SearchResult, Usage, UsageMap } from './types';
import { DIRECT_OPEN_ID } from './constants';
import { boundaryMatch, BoundaryMatch } from './boundaryMatch';

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

/** The most used commands, best first. Commands never run are left out, so a new install lists none. */
export function frequentCommands(
    commands: Command[],
    usage: UsageMap,
    maxResults: number,
    now: number = Date.now(),
): SearchResult[] {
    return commands
        .filter((command) => usage[command.id])
        .map((command) => ({ command, boost: usageBoost(usage[command.id], now) }))
        .sort((a, b) => b.boost - a.boost)
        .slice(0, maxResults)
        .map(({ command }) => ({ command, score: 0, matches: [] }));
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
    // Re-ranking by history needs every match, not just the first maxResults. Scattered one-letter
    // hits are kept and highlighted like longer runs, so a result never looks unexplained.
    const ranked = new Map<string, SearchResult>();
    for (const r of fuse.search(query)) {
        ranked.set(r.item.id, {
            command: r.item,
            score: r.score ?? 0,
            matches: r.matches?.map((m) => ({ indices: [...m.indices], key: m.key ?? '' })) ?? [],
        });
    }
    // Abbreviations such as "vscode" are invisible to Fuse; add them, or lift them when this fits better
    for (const command of commands) {
        const hit = boundaryMatch(command.name, query);
        if (!hit) continue;
        const score = boundaryScore(hit, query);
        const existing = ranked.get(command.id);
        if (existing && existing.score <= score) continue;
        ranked.set(command.id, {
            command,
            score,
            matches: [{ indices: hit.indices, key: 'name' }],
        });
    }
    return [...ranked.values()]
        .map((r) => ({ r, rank: r.score - usageBoost(usage[r.command.id], now) }))
        .sort((a, b) => a.rank - b.rank)
        .slice(0, maxResults)
        .map(({ r }) => r);
}

// A word-start match scores like a good Fuse match, tighter (fewer words) first. It stays below
// 0.05 so that history, which is capped at that much, cannot reorder it against a literal hit.
function boundaryScore(hit: BoundaryMatch, query: string): number {
    const length = Math.max(1, query.replace(/\s+/g, '').length);
    return 0.02 + 0.03 * Math.min(1, hit.words / length);
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
