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
    runAsAdmin?: boolean;
    openWith?: string;
    openArgs?: string;
}

/** How often and how recently a command was run, keyed by the `command` string (ids of external commands do not last). */
export interface Usage {
    count: number;
    lastUsed: number; // Unix time in milliseconds
}

export type UsageMap = Record<string, Usage>;

export interface ApplicationSettings {
    enabled: boolean;
    directories: string[];
    extensions: string[];
}

/** The browsers bookmark.rs can read. */
export const BROWSERS = ['brave', 'chrome', 'edge'] as const;

export interface BookmarkSettings {
    enabled: boolean;
    browser: (typeof BROWSERS)[number];
}

export interface MarkdownSettings {
    enabled: boolean;
    paths: string[];
}

/** 登録フォルダの下のファイル・フォルダ(Local Path)。src-tauri/src/settings/domain.rs と揃える */
export interface LocalPathSettings {
    enabled: boolean;
    paths: string[];
    depth: number;
    /** ドットなし。`*` はすべてのファイル、空ならファイルなし */
    extensions: string[];
    includeFolders: boolean;
    excludeNames: string[];
}

export interface Settings {
    maxResults: number;
    fuzzyThreshold: number;
    bookmarks: BookmarkSettings;
    applications: ApplicationSettings;
    markdown: MarkdownSettings;
    localPaths: LocalPathSettings;
    ignored: string[];
    /** Aliases of external commands, keyed by their `command` string. */
    aliases: Record<string, string>;
    theme: string;
    hotkey: string;
    reloadIntervalMinutes: number;
    usageHistory: boolean;
    frequentFirst: boolean;
}

/** A problem found while the window may have been hidden, from the backend's `get_notices`. */
export interface Notice {
    /** What raised it; a source holds one notice at a time. */
    source: string;
    level: 'error' | 'warning';
    message: string;
}

export interface SearchResult {
    command: Command;
    score: number;
    matches: Array<{
        indices: readonly [number, number][];
        key: string;
    }>;
}
