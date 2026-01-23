import { Command, PromptParseResult, SearchResult } from '../types';
import { SearchEngine } from './SearchEngine';

/**
 * PromptProcessor handles parsing and processing of user input with prompt support.
 */
export class PromptProcessor {
  private searchEngine: SearchEngine;
  private commandList: Command[] = [];

  constructor(searchEngine: SearchEngine) {
    this.searchEngine = searchEngine;
  }

  updateCommands(commands: Command[]): void {
    this.commandList = commands;
    this.searchEngine.updateCommands(commands);
  }

  parseInput(input: string): PromptParseResult {
    const trimmedInput = input.trim();
    
    if (!trimmedInput) {
      return { query: '', args: [] };
    }

    const inputParts = trimmedInput.split(/\s+/);
    
    if (inputParts.length === 1) {
      return { query: inputParts[0], args: [] };
    }

    const [potentialPrompt, ...remainingParts] = inputParts;
    
    if (this.isPromptRegistered(potentialPrompt)) {
      return {
        prompt: potentialPrompt,
        query: remainingParts.join(' '),
        args: remainingParts,
      };
    }

    return {
      query: trimmedInput,
      args: inputParts,
    };
  }

  processSearch(input: string, maxResults: number = 10): SearchResult[] {
    const parsedInput = this.parseInput(input);

    if (parsedInput.prompt) {
      return this.executePromptSearch(parsedInput.prompt, parsedInput.query, maxResults);
    }
    
    return this.searchEngine.search(parsedInput.query, maxResults);
  }

  getCommandsWithPrompt(prompt: string): Command[] {
    return this.commandList.filter(cmd => cmd.prompt === prompt);
  }

  getAllPrompts(): string[] {
    const prompts = this.commandList
      .map(cmd => cmd.prompt)
      .filter((prompt): prompt is string => Boolean(prompt));
    
    return [...new Set(prompts)].sort();
  }

  private isPromptRegistered(prompt: string): boolean {
    return this.commandList.some(cmd => cmd.prompt === prompt);
  }

  private executePromptSearch(prompt: string, query: string, maxResults: number): SearchResult[] {
    const promptCommands = this.getCommandsWithPrompt(prompt);

    if (promptCommands.length === 0) {
      // プロンプトが見つからない場合は、元の入力全体で検索
      return this.searchEngine.search(`${prompt} ${query}`, maxResults);
    }

    // プロンプトが見つかった場合は、そのコマンドを返す（引数は後でプレースホルダー置換される）
    return promptCommands.slice(0, maxResults).map(command => ({
      command,
      score: 0,
      matches: [],
    }));
  }
}