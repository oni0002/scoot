export interface Command {
    id: string;
    name: string;
    category: string;
    source?: string;
    command: string;
    description: string;
    alias?: string;
    workingDir?: string;
    showWindow?: boolean;
}

/** How often and how recently a command was run, keyed by command id. */
export interface Usage {
    count: number;
    lastUsed: number; // Unix time in milliseconds
}

export type UsageMap = Record<string, Usage>;

export interface ApplicationConfig {
    enabled: boolean;
    directories: string[];
    extensions: string[];
}

export interface BookmarkConfig {
    enabled: boolean;
    browser: 'brave' | 'chrome' | 'edge'; // the browsers bookmark.rs can read
}

export interface MarkdownConfig {
    enabled: boolean;
    paths: string[];
}

export interface Config {
    maxResults: number;
    fuzzyThreshold: number;
    bookmarks: BookmarkConfig;
    applications: ApplicationConfig;
    markdown: MarkdownConfig;
    ignored: string[];
    /** Aliases of external commands, keyed by their `command` string. */
    aliases: Record<string, string>;
    theme: string;
    hotkey: string;
    reloadIntervalMinutes: number;
    usageHistory: boolean;
    frequentFirst: boolean;
}

export interface SearchResult {
    command: Command;
    score: number;
    matches: Array<{
        indices: readonly [number, number][];
        key: string;
    }>;
}
