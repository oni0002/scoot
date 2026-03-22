export interface Command {
    id: string;
    name: string;
    category: string;
    command: string;
    description: string;
    prompt?: string;
    workingDir?: string;
    showWindow?: boolean;
}

export type Commands = Command[];

export interface ApplicationConfig {
    enabled: boolean;
    directories: string[];
}

export interface BookmarkConfig {
    enabled: boolean;
    browser: 'brave' | 'chrome' | 'firefox' | 'edge';
    prompt?: string;
    refreshIntervalMinutes: number;
}

export interface Config {
    maxResults: number;
    fuzzyThreshold: number;
    bookmarks: BookmarkConfig;
    applications: ApplicationConfig;
    theme: string;
}
