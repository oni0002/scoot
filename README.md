# Scoot

A simple, fast, and keyboard-centric command launcher for Windows.

- **Global Hotkey** - Launch instantly with `Alt+Space` (default).
- **Fuzzy Search** - Find what you need quickly with fuzzy matching.
- **Applications** - Automatically scans your Start Menu for installed programs.
- **Bookmarks** - Search your browser bookmarks (Chrome, Brave, Edge).
- **Custom Commands** - Open files, URLs, or run shell scripts effortlessly.
- **Prompt Mode** - Pass dynamic arguments to your commands (e.g., `g react` opens `https://google.com/search?q=react`).

## Quick Start

1. **Launch**: Press `Alt + Space` to show the Scoot window.
2. **Search**: Start typing to find applications, bookmarks, or custom commands.
3. **Navigate**: Use `Tab` / `Shift+Tab` (or `Up`/`Down` arrows) to select a result.
4. **Execute**: Press `Enter` to run the selected command.

## Keybindings

| Key                        | Action                                      |
| -------------------------- | ------------------------------------------- |
| `Alt + Space` (Default)    | Toggle Scoot window visibility              |
| `Esc`                      | Clear input / Close window                  |
| `Enter`                    | Execute the selected command                |
| `↑` / `↓`                  | Navigate search results                     |
| `Tab` / `Shift + Tab`      | Navigate search results (Next / Previous)   |
| `Ctrl + N` / `Ctrl + P`    | Navigate search results (Emacs style)       |

## Configuration

Settings are defined in `config.json`. You can open this file via the `Open config.json` preset command or from the 3-dot menu. 
Scoot uses `camelCase` for configuration keys.

```json
{
  "maxResults": 30,
  "fuzzyThreshold": 0.4,
  "theme": "dark",
  "hotkey": "Alt+Space",
  "bookmarks": {
    "enabled": true,
    "browser": "brave",
    "prompt": "b",
    "refreshIntervalMinutes": 60
  },
  "applications": {
    "enabled": true,
    "directories": [
      "%APPDATA%\\Microsoft\\Windows\\Start Menu\\Programs",
      "C:\\ProgramData\\Microsoft\\Windows\\Start Menu\\Programs"
    ],
    "extensions": ["lnk"]
  }
}
```

The `config.json` and `commands.json` files are automatically generated in `%APPDATA%\scoot\` on first launch.

## User Custom Commands

You can add custom commands to expand Scoot's capabilities.

### Adding a Command
1. Click the 3-dot menu and select **Add Command** (or use the `Add Command` preset).
2. Fill in the required fields:
   - **Name**: Display name (e.g., `Google Search`)
   - **Description**: What the command does (e.g., `Search the web using Google`)
   - **Category**: `URL`, `File`, `Command`, or `Custom`
   - **Command**: The target path, URL, or shell command to execute
   - **Prompt**: (Optional) A short prefix string used to trigger this command and accept arguments (e.g., `g`)

### Using Prompt Arguments
You can define placeholders in your `Command` field that will be replaced by the arguments you type after the prompt.

- `{$n}`: Replaced by the n-th argument.
- `{$*}`: Replaced by all arguments combined.

**Example: Google Search**
- **Prompt**: `g`
- **Command**: `https://www.google.com/search?q={$*}`
- **Usage**: Type `g react hooks` in Scoot.
- **Result**: Opens `https://www.google.com/search?q=react hooks` in your browser.