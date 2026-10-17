import { emit } from '@tauri-apps/api/event';
import { mockIPC, mockWindows } from '@tauri-apps/api/mocks';
import { Command, Config } from '../types';

// Dev-only stand-in for the Rust backend so the UI can be opened in a plain browser
// (pnpm dev -> http://localhost:1420) for visual review and automated UI checks.
// Not part of the production bundle: main.tsx imports it only under import.meta.env.DEV.

type Args = Record<string, unknown>;
type Handler = (args: Args) => unknown;

const cmd = (
    id: string,
    name: string,
    category: string,
    source: string,
    command: string,
    description: string,
    extra: Partial<Command> = {},
): Command => ({ id, name, category, source, command, description, ...extra });

const userCommands = (): Command[] => [
    cmd(
        'u1',
        'Duckduckgo search',
        'url',
        'user',
        'https://duckduckgo.com/search?q={$1}',
        'ddg search',
        { alias: 'd' },
    ),
    cmd('u2', 'download', 'file', 'user', 'C:\\Users\\demo\\Downloads', ''),
    cmd('u3', 'GitHub', 'url', 'user', 'https://github.com', 'Code hosting'),
    cmd('u4', 'Deploy staging', 'command', 'user', 'pnpm deploy:staging', 'Build and push', {
        workingDir: 'C:\\work\\app',
        showWindow: true,
    }),
    // Fails on purpose: exercises the execution-error path
    cmd('u5', 'Missing report', 'file', 'user', 'C:\\does\\not\\exist.pdf', 'Always fails'),
];

const externalCommands = (): Command[] => [
    cmd('a1', 'Discord', 'file', 'application', 'C:\\Apps\\Discord.lnk', 'Discord Inc'),
    cmd('a2', 'Drax', 'file', 'application', 'C:\\Apps\\Drax.lnk', 'Programs'),
    cmd(
        'a3',
        'Disk Cleanup',
        'file',
        'application',
        'C:\\Apps\\cleanmgr.lnk',
        'Administrative Tools',
    ),
    cmd('a4', 'drawio (Ubuntu)', 'file', 'application', 'C:\\Apps\\drawio.lnk', 'Ubuntu'),
    cmd('a5', 'Node.js', 'file', 'application', 'C:\\Apps\\node.lnk', 'Node.js'),
    cmd('a6', 'Zed', 'file', 'application', 'C:\\Apps\\Zed.lnk', 'Zed'),
    cmd('b1', 'daisyUI Documentation', 'url', 'bookmark', 'https://daisyui.com/docs/', ''),
    cmd('b2', 'Tauri v2 Guide', 'url', 'bookmark', 'https://v2.tauri.app/', ''),
    cmd('s1', 'Add Command', 'scoot', 'scoot', 'scoot://add-command', 'Add a new command'),
    cmd('s2', 'Open Settings', 'scoot', 'scoot', 'scoot://open-settings', 'Open settings'),
    cmd('s3', 'Reload', 'scoot', 'scoot', 'scoot://reload', 'Reload commands and configuration'),
];

/** `?theme=<name>` picks the theme, so any theme can be opened by URL while evaluating the UI. */
const previewTheme = (): string | null =>
    typeof window === 'undefined' ? null : new URLSearchParams(window.location.search).get('theme');

const defaultConfig = (): Config => ({
    maxResults: 10,
    fuzzyThreshold: 0.5,
    bookmarks: { enabled: true, browser: 'brave' },
    applications: {
        enabled: true,
        directories: [
            '%APPDATA%\\Microsoft\\Windows\\Start Menu\\Programs',
            '%PROGRAMDATA%\\Microsoft\\Windows\\Start Menu\\Programs',
        ],
        extensions: ['lnk'],
    },
    markdown: { enabled: false, paths: [] },
    ignored: [
        'https://example.com/old-bookmark',
        'C:\\ProgramData\\Microsoft\\Windows\\Start Menu\\Programs\\Administrative Tools\\dfrgui.lnk',
    ],
    theme: previewTheme() ?? 'dark',
    hotkey: 'Alt+Space',
    reloadIntervalMinutes: 30,
});

export function createMockHandlers(): Record<string, Handler> {
    let users = userCommands();
    let config = defaultConfig();
    let nextId = 100;
    const visibleExternal = () =>
        externalCommands().filter((c) => !config.ignored.includes(c.command));

    return {
        get_all_commands: () => [...users, ...visibleExternal()],
        add_command: ({ command }) => {
            const c = command as Command;
            if (c.alias && users.some((u) => u.alias === c.alias)) {
                throw new Error(`Alias '${c.alias}' is already used by another command.`);
            }
            const id = `u${nextId++}`;
            users = [...users, { ...c, id, source: 'user' }];
            return id;
        },
        update_command: ({ command }) => {
            const c = command as Command;
            if (c.alias && users.some((u) => u.alias === c.alias && u.id !== c.id)) {
                throw new Error(`Alias '${c.alias}' is already used by another command.`);
            }
            users = users.map((u) => (u.id === c.id ? { ...c, source: 'user' } : u));
        },
        delete_command: ({ id }) => {
            users = users.filter((u) => u.id !== id);
        },
        execute_command: async ({ id }) => {
            const found = [...users, ...externalCommands()].find((c) => c.id === id);
            if (!found) throw new Error(`Command not found: ${String(id)}`);
            if (found.id === 'u5') {
                throw new Error(
                    `File not found: '${found.command}'. Please check if the file exists and the path is correct.`,
                );
            }
            if (found.command === 'scoot://open-settings') {
                await emit('open-settings-dialog');
                return { keepWindowOpen: true };
            }
            if (found.command === 'scoot://add-command') {
                await emit('open-add-command-dialog');
                return { keepWindowOpen: true };
            }
            if (found.command === 'scoot://reload') return { keepWindowOpen: true };
            return { keepWindowOpen: false };
        },
        open_target: () => ({ keepWindowOpen: false }),
        reload_all: () => undefined,
        hide_window: () => undefined,
        open_commands_json: () => undefined,
        open_config_json: () => undefined,
        quit_app: () => undefined,
        enter_modal: () => undefined,
        leave_modal: () => undefined,
        start_hotkey_capture: () => undefined,
        stop_hotkey_capture: () => undefined,
        get_config: () => config,
        save_config: ({ config: next }) => {
            config = next as Config;
        },
        ignore_command: ({ commandStr }) => {
            config = { ...config, ignored: [...config.ignored, String(commandStr)] };
        },
        unignore_command: ({ commandStr }) => {
            config = { ...config, ignored: config.ignored.filter((c) => c !== commandStr) };
        },
        'plugin:app|version': () => '0.2.10',
        'plugin:dialog|message': () => undefined,
        'plugin:dialog|open': () => null,
    };
}

export function installMockBackend(): void {
    const handlers = createMockHandlers();
    mockWindows('main');
    mockIPC(
        (command, payload) => {
            const handler = handlers[command];
            if (!handler) throw new Error(`mock backend: unhandled command "${command}"`);
            return handler((payload ?? {}) as Args);
        },
        { shouldMockEvents: true },
    );
}
