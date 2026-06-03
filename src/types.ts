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

export interface ApplicationConfig {
    enabled: boolean;
    directories: string[];
    extensions: string[];
}

export interface BookmarkConfig {
    enabled: boolean;
    browser: 'brave' | 'chrome' | 'firefox' | 'edge';
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
    theme: string;
    hotkey: string;
    reloadIntervalMinutes: number;
}

export interface SearchResult {
    command: Command;
    score: number;
    matches: Array<{
        indices: readonly [number, number][];
        key: string;
    }>;
}
