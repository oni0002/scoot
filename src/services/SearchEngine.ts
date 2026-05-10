import Fuse, { IFuseOptions } from 'fuse.js';
import { Command, SearchResult } from '../types';

/**
 * SearchEngine provides fuzzy search functionality for commands using Fuse.js.
 * Supports both general search and category-specific search with configurable options.
 */
export class SearchEngine {
  private fuseInstance: Fuse<Command>;
  private commandList: Command[] = [];
  private fuseOptions: IFuseOptions<Command>;

  private static readonly DEFAULT_FUSE_OPTIONS: IFuseOptions<Command> = {
    keys: [
      { name: 'name', weight: 0.7 },
      { name: 'prompt', weight: 0.5 },
    ],
    threshold: 0.5,
    includeScore: true,
    includeMatches: true,
    minMatchCharLength: 1,
    ignoreLocation: false,
    findAllMatches: true,
    useExtendedSearch: false,
  };

  constructor(commands: Command[] = [], threshold: number = 0.5) {
    this.commandList = commands;
    this.fuseOptions = {
      ...SearchEngine.DEFAULT_FUSE_OPTIONS,
      threshold: this.clampThreshold(threshold),
    };
    this.fuseInstance = new Fuse(commands, this.fuseOptions);
  }

  updateCommands(commands: Command[]): void {
    this.commandList = commands;
    this.fuseInstance = new Fuse(commands, this.fuseOptions);
  }

  updateThreshold(threshold: number): void {
    this.fuseOptions = {
      ...this.fuseOptions,
      threshold: this.clampThreshold(threshold),
    };
    this.fuseInstance = new Fuse(this.commandList, this.fuseOptions);
  }

  search(query: string, maxResults: number = 10): SearchResult[] {
    if (!query.trim()) {
      return this.createDefaultResults(maxResults);
    }

    const fuseResults = this.fuseInstance.search(query, { limit: maxResults });
    return this.transformFuseResults(fuseResults);
  }

  private clampThreshold(threshold: number): number {
    return Math.max(0.0, Math.min(1.0, threshold));
  }

  private createDefaultResults(maxResults: number): SearchResult[] {
    return this.commandList.slice(0, maxResults).map(command => ({
      command,
      score: 0,
      matches: [],
    }));
  }

  private transformFuseResults(fuseResults: any[]): SearchResult[] {
    return fuseResults.map(result => ({
      command: result.item,
      score: result.score || 0,
      matches: result.matches?.map((match: any) => ({
        indices: match.indices,
        key: match.key || '',
      })) || [],
    }));
  }
}