import { Command } from './models';

export interface SearchResult {
    command: Command;
    score: number;
    matches: Array<{
        indices: readonly [number, number][];
        key: string;
    }>;
}

export enum CommandType {
    Url = 'Url',
    LocalFile = 'LocalFile',
    SystemCommand = 'SystemCommand',
}

export interface PromptParseResult {
    prompt?: string;
    query: string;
    args: string[];
}
