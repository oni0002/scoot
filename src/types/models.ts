export interface Command {
    id: string;
    name: string;
    category: string;
    command: string;
    description: string;
    prompt?: string;
    working_dir?: string;
    show_window?: boolean;
    is_editable: boolean;
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
    refresh_interval_minutes: number;
}

export interface Config {
    max_results: number;
    fuzzy_threshold: number;
    bookmarks: BookmarkConfig;
    applications: ApplicationConfig;
    theme: string;
}
