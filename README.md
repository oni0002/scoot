# Scoot

A simple, fast, and keyboard-centric command launcher for Windows.

- **Global Hotkey** - Launch instantly with `Alt+Space` (default).
- **Fuzzy Search** - Find what you need quickly with fuzzy matching.
- **Applications** - Automatically scans your Start Menu for installed programs.
- **Bookmarks** - Search your browser bookmarks (Chrome, Brave, Edge).
- **Markdown Links** - Search your markdown files for links.
- **Custom Commands** - Open files, URLs, or run shell scripts effortlessly.
- **Argument Mode** - Pass dynamic arguments to your commands via an alias (e.g., `g react` opens `https://google.com/search?q=react`).

## Quick Start

1. **Install**: Scoot is a portable application. Simply place the `.exe` file in any directory of your choice and run it.
2. **Launch**: Press `Alt + Space` to show the Scoot window.
3. **Search**: Start typing to find applications, bookmarks, or custom commands.
4. **Navigate**: Use `Tab` / `Shift+Tab` (or `Up`/`Down` arrows) to select a result.
5. **Execute**: Press `Enter` to run the selected command.

## Keybindings

| Key                                      | Action                                                        |
| ---------------------------------------- | ------------------------------------------------------------- |
| `Alt + Space` (Default)                  | Toggle Scoot window visibility                                |
| `Esc`                                    | Clear input / Close window                                    |
| `Enter`                                  | Execute the selected command                                  |
| `↑`/`↓`, `Tab`/`Shift+Tab`, `Ctrl+N`/`P` | Navigate search results                                       |
| `Ctrl+C`                                 | Copy selected command to clipboard (when no text is selected) |

## Settings

Open the Settings screen from the 3-dot menu or via the `Open Settings` preset command. Settings are saved automatically on close.

**General**

| Setting         | Description                                                          | Default     |
| --------------- | -------------------------------------------------------------------- | ----------- |
| Theme           | `system` (follows the OS light/dark setting), `light` or `dark`      | `system`    |
| Hotkey          | Global shortcut to show Scoot                                        | `Alt+Space` |
| Max Results     | Maximum number of search results shown                               | `10`        |
| Fuzzy Threshold | How closely queries must match (0.0 = exact, 1.0 = anything)         | `0.5`       |
| Reload Interval | How often sources are reloaded, in minutes                           | `30`        |
| Ignored         | Commands hidden from search results. Add via the ⋮ menu on a result. | `[]`        |
| Frequent first  | List the commands you run most before you type (needs usage history) | `true`      |
| Launch at login | Button that adds or removes a shortcut in the Windows Startup folder | -           |

**Bookmarks**

| Setting | Description                     | Default |
| ------- | ------------------------------- | ------- |
| Enabled | Load bookmarks from the browser | `true`  |
| Browser | Browser to read bookmarks from  | `brave` |

**Applications**

| Setting     | Description                                | Default          |
| ----------- | ------------------------------------------ | ---------------- |
| Enabled     | Scan directories for application shortcuts | `true`           |
| Directories | Folders to scan for shortcuts              | Start Menu paths |
| Extensions  | File extensions treated as applications    | `lnk`            |

**Markdown**

| Setting | Description                                         | Default |
| ------- | --------------------------------------------------- | ------- |
| Enabled | Extract links from Markdown files                   | `false` |
| Files   | Markdown files to read URL and file-path links from | `[]`    |

Relative file paths in a Markdown file are taken from the folder that file is in. Links with other schemes (`javascript:`, `obsidian://`, ...) are skipped.

The `settings.json` and `commands.json` files are stored in the same directory as the `.exe` file. So is `usage.json`, the run history that nudges frequently and recently used commands up in search results; turn it off or clear it under Settings > General.

Up to 0.2.x the settings file was called `config.json`. The first start of a newer version renames it to `settings.json`; if both exist, `settings.json` is used. Going back to an older version creates a fresh `config.json`, so copy your values over by hand.

## User Custom Commands

You can add custom commands to expand Scoot's capabilities.

### Adding a Command

1. Click the 3-dot menu and select **Add Command** (or use the `Add Command` preset).
2. Fill in the required fields:
    - **Name**: Display name of the command. _This name is used for fuzzy searching._ (e.g., `Google Search`)
    - **Description**: What the command does (e.g., `Search the web using Google`)
    - **Category**: `URL`, `File`, or `Command`
    - **Command**: The target path, URL, or shell command to execute
    - **Alias**: A short name for the command (e.g., `g`). Optional, except for commands that take arguments. Search matches it like the name.

Applications, bookmarks and other commands that Scoot finds for you can have an alias too: open the ⋮ menu on a result (or press `Ctrl+L`) and choose **Alias**. It is saved in `settings.json`.

### Using Alias Arguments

You can define placeholders in your `Command` field that will be replaced by the arguments you type after the alias.

- `{$n}`: Replaced by the n-th argument.
- `{$*}`: Replaced by all arguments combined.

Example: Google Search

- **Alias**: `g`
- **Command**: `https://www.google.com/search?q={$*}`
- **Usage**: Type `g react` in Scoot.
- **Result**: Opens `https://www.google.com/search?q=react` in your browser.
