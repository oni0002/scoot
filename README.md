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
2. **Search**: Start typing to find applications, bookmarks, or custom commands.
3. **Navigate**: Use `Tab` / `Shift+Tab` (or `Up`/`Down` arrows) to select a result.
4. **Execute**: Press `Enter` to run the selected command.

## Keybindings

| Key                        | Action                                      |
| -------------------------- | ------------------------------------------- |
| `Alt + Space` (Default)    | Toggle Scoot window visibility              |
| `Esc`                      | Clear input / Close window                  |
| `Enter`                    | Execute the selected command                |
| `↑`/`↓`, `Tab`/`Shift+Tab`, `Ctrl+N`/`P`| Navigate search results           |
| `Ctrl+C`                   | Copy selected command to clipboard (when no text is selected) |

## Configuration

Open the Settings screen from the 3-dot menu or via the `Open Settings` preset command. Settings are saved automatically on close.

**General**

| Setting | Description | Default |
| --- | --- | --- |
| Theme | UI color theme | `dark` |
| Hotkey | Global shortcut to show Scoot | `Alt+Space` |
| Max Results | Maximum number of search results shown | `30` |
| Fuzzy Threshold | How strictly queries must match (0.0 = permissive, 1.0 = strict) | `0.4` |
| Reload Interval | How often sources are reloaded, in minutes | `60` |
| Ignored | Commands hidden from search results. Add via the ⋯ menu on a result. | `[]` |

**Bookmarks**

| Setting | Description | Default |
| --- | --- | --- |
| Enabled | Load bookmarks from the browser | `true` |
| Browser | Browser to read bookmarks from | `brave` |

**Applications**

| Setting | Description | Default |
| --- | --- | --- |
| Enabled | Scan directories for application shortcuts | `true` |
| Directories | Folders to scan for shortcuts | Start Menu paths |
| Extensions | File extensions treated as applications | `lnk` |

**Markdown**

| Setting | Description | Default |
| --- | --- | --- |
| Enabled | Extract links from Markdown files | `false` |
| Files | Markdown files to read URL and file-path links from | `[]` |

The `config.json` and `commands.json` files are stored in the same directory as the `.exe` file.

## User Custom Commands

You can add custom commands to expand Scoot's capabilities.

### Adding a Command

1. Click the 3-dot menu and select **Add Command** (or use the `Add Command` preset).
2. Fill in the required fields:
   - **Name**: Display name of the command. *This name is used for fuzzy searching.* (e.g., `Google Search`)
   - **Description**: What the command does (e.g., `Search the web using Google`)
   - **Category**: `URL`, `File`, or `Command`
   - **Command**: The target path, URL, or shell command to execute
   - **Alias**: (Optional) A short prefix string used to trigger this command and accept arguments (e.g., `g`)

### Using Alias Arguments

You can define placeholders in your `Command` field that will be replaced by the arguments you type after the alias.

- `{$n}`: Replaced by the n-th argument.
- `{$*}`: Replaced by all arguments combined.

Example: Google Search

- **Alias**: `g`
- **Command**: `https://www.google.com/search?q={$*}`
- **Usage**: Type `g react` in Scoot.
- **Result**: Opens `https://www.google.com/search?q=react` in your browser.
