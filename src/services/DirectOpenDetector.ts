import { Command } from '../types';
import { DIRECT_OPEN_ID } from '../constants';

export class DirectOpenDetector {
  // Regex patterns
  private static URL_PATTERN = /^(https?|ftp|file|mailto|scoot):\/\/?/i;
  private static UNC_PATH_PATTERN = /^\\\\/i;
  private static WIN_ABS_PATH_PATTERN = /^[A-Za-z]:\\/i;
  private static UNIX_ABS_PATH_PATTERN = /^(\/|~\/)/i;

  /**
   * Detects if the given query is a URL or a file path.
   * If it is, returns a dummy Command object to open it directly.
   */
  public static detect(query: string): Command | null {
    const trimmed = query.trim();
    if (!trimmed) return null;

    const isUrl = this.URL_PATTERN.test(trimmed);
    const isWinAbsPath = this.WIN_ABS_PATH_PATTERN.test(trimmed);
    const isUncPath = this.UNC_PATH_PATTERN.test(trimmed);
    const isUnixAbsPath = this.UNIX_ABS_PATH_PATTERN.test(trimmed);

    if (isUrl || isWinAbsPath || isUncPath || isUnixAbsPath) {
      const category = isUrl ? 'url' : 'file';

      return {
        id: DIRECT_OPEN_ID,
        name: `Open: ${trimmed}`,
        category: category,
        source: 'user',
        command: trimmed,
        description: `Open ${category}`
      };
    }

    return null;
  }
}
