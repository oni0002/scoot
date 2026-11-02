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
    /** Local Path で、この項目を一覧にした登録フォルダの Alias(実行時のみ) */
    scopes?: string[];
    /** 通常の検索には出さず、`scopes` の Alias からだけ探せる */
    scopedOnly?: boolean;
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

/** 登録フォルダ1つと、その下のどこまで・何を一覧にするか。src-tauri/src/settings/domain.rs と揃える */
export interface LocalPathEntry {
    path: string;
    depth: number;
    /** ドットなし。`*` はすべてのファイル、空ならファイルなし */
    extensions: string[];
    includeFolders: boolean;
    /** `<alias> <keyword>` でこのフォルダの下だけを探す。Command の Alias とは別の名前空間 */
    alias?: string;
    /** 通常の検索には出さない。`alias` が要る */
    aliasOnly: boolean;
}

/** 登録フォルダの下のファイル・フォルダ(Local Path) */
export interface LocalPathSettings {
    enabled: boolean;
    entries: LocalPathEntry[];
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
