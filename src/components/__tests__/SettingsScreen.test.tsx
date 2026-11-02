import { render, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import '@testing-library/jest-dom';
import { SettingsScreen } from '../SettingsScreen';
import { SettingsProvider, useSettingsContext } from '../../context/SettingsContext';
import { NotificationProvider } from '../../context/NotificationContext';
import { TauriAPI } from '../../tauri';
import { createMockHandlers } from '../../dev/mockBackend';
import { Settings } from '../../types';

jest.mock('@tauri-apps/api/event', () => ({ listen: jest.fn().mockResolvedValue(() => {}) }));
jest.mock('@tauri-apps/plugin-dialog', () => ({ open: jest.fn() }));
jest.mock('../../tauri');

// jsdom does not implement <dialog> modals
beforeAll(() => {
    HTMLDialogElement.prototype.showModal = function () {
        this.setAttribute('open', '');
    };
    HTMLDialogElement.prototype.close = function () {
        if (!this.hasAttribute('open')) return;
        this.removeAttribute('open');
        this.dispatchEvent(new Event('close'));
    };
});

const settings = createMockHandlers().get_settings({}) as Settings;

const Loaded = ({
    onBack,
    onClearHistory,
}: {
    onBack: () => void;
    onClearHistory: () => Promise<void>;
}) => {
    const { settings: loaded, loadSettings } = useSettingsContext();
    if (!loaded) {
        void loadSettings();
        return null;
    }
    return <SettingsScreen onBack={onBack} onClearHistory={onClearHistory} />;
};

const renderScreen = (
    onBack = jest.fn(),
    onClearHistory = jest.fn().mockResolvedValue(undefined),
) => {
    render(
        <NotificationProvider>
            <SettingsProvider>
                <Loaded onBack={onBack} onClearHistory={onClearHistory} />
            </SettingsProvider>
        </NotificationProvider>,
    );
    return onBack;
};

beforeEach(() => {
    jest.mocked(TauriAPI.getSettings).mockResolvedValue(settings);
    jest.mocked(TauriAPI.saveSettings).mockReset();
    jest.mocked(TauriAPI.getStartupShortcut).mockResolvedValue(false);
    jest.mocked(TauriAPI.setStartupShortcut).mockReset();
});

describe('SettingsScreen closing', () => {
    it('closes without saving when nothing changed', async () => {
        const onBack = renderScreen();
        await userEvent.click(await screen.findByRole('button', { name: 'Back' }));
        expect(onBack).toHaveBeenCalled();
        expect(TauriAPI.saveSettings).not.toHaveBeenCalled();
    });

    it('saves edits, says so, and closes', async () => {
        jest.mocked(TauriAPI.saveSettings).mockResolvedValue();
        const onBack = renderScreen();
        await userEvent.selectOptions(await screen.findByLabelText('Theme'), 'light');
        await userEvent.click(screen.getByRole('button', { name: 'Back' }));
        await waitFor(() => expect(onBack).toHaveBeenCalled());
        expect(TauriAPI.saveSettings).toHaveBeenCalledWith(
            expect.objectContaining({ theme: 'light' }),
        );
        expect(screen.getByText('Settings saved')).toBeInTheDocument();
    });

    it('stays open and shows why when saving fails', async () => {
        jest.mocked(TauriAPI.saveSettings).mockRejectedValue(
            'Validation error: maxResults must be between 1 and 100 (got 0).',
        );
        const onBack = renderScreen();
        await userEvent.selectOptions(await screen.findByLabelText('Theme'), 'light');
        await userEvent.click(screen.getByRole('button', { name: 'Back' }));
        expect(await screen.findByRole('alert')).toHaveTextContent(
            'Could not save settings: maxResults must be between 1 and 100 (got 0).',
        );
        expect(onBack).not.toHaveBeenCalled();

        await userEvent.click(screen.getByRole('button', { name: 'Discard and close' }));
        expect(onBack).toHaveBeenCalled();
    });
});

describe('SettingsScreen Esc', () => {
    it('closes the screen', async () => {
        const onBack = renderScreen();
        await screen.findByLabelText('Theme');
        await userEvent.keyboard('{Escape}');
        expect(onBack).toHaveBeenCalled();
    });

    it('saves once when Esc is pressed twice', async () => {
        let finish: () => void = () => {};
        jest.mocked(TauriAPI.saveSettings).mockImplementation(
            () => new Promise<void>((resolve) => (finish = resolve)),
        );
        const onBack = renderScreen();
        await userEvent.selectOptions(await screen.findByLabelText('Theme'), 'light');

        await userEvent.keyboard('{Escape}{Escape}');
        finish();
        await waitFor(() => expect(onBack).toHaveBeenCalledTimes(1));
        expect(TauriAPI.saveSettings).toHaveBeenCalledTimes(1);
    });

    it('only closes the confirmation when one is open', async () => {
        const onBack = renderScreen();
        await userEvent.click(await screen.findByRole('button', { name: 'Clear' }));
        expect(screen.getByRole('dialog')).toBeInTheDocument();
        await userEvent.keyboard('{Escape}');
        expect(onBack).not.toHaveBeenCalled();
    });

    it('only closes the extension field when it has focus', async () => {
        const onBack = renderScreen();
        const apps = await screen.findByRole('region', { name: 'Applications' });
        await userEvent.click(within(apps).getByRole('button', { name: 'Add extension' }));
        await userEvent.type(screen.getByLabelText('New extension'), '{Escape}');
        expect(screen.queryByLabelText('New extension')).not.toBeInTheDocument();
        expect(onBack).not.toHaveBeenCalled();
    });
});

describe('SettingsScreen sections', () => {
    it('disables a sources controls while its switch is off', async () => {
        renderScreen();
        const browser = await screen.findByLabelText('Browser');
        expect(browser).toBeEnabled();
        await userEvent.click(screen.getByRole('checkbox', { name: 'Enable Bookmarks' }));
        expect(browser).toBeDisabled();
        await userEvent.click(screen.getByRole('checkbox', { name: 'Enable Bookmarks' }));
        expect(browser).toBeEnabled();
    });

    it('adds a typed extension without its dot and saves it', async () => {
        jest.mocked(TauriAPI.saveSettings).mockResolvedValue();
        renderScreen();
        const apps = await screen.findByRole('region', { name: 'Applications' });
        await userEvent.click(within(apps).getByRole('button', { name: 'Add extension' }));
        await userEvent.type(within(apps).getByLabelText('New extension'), '.url{Enter}');
        expect(within(apps).getByText('.url')).toBeInTheDocument();

        await userEvent.click(screen.getByRole('button', { name: 'Back' }));
        await waitFor(() =>
            expect(TauriAPI.saveSettings).toHaveBeenCalledWith(
                expect.objectContaining({
                    applications: expect.objectContaining({ extensions: ['lnk', 'url'] }),
                }),
            ),
        );
    });

    it('lets plain Tab leave the hotkey field instead of trapping focus', async () => {
        renderScreen();
        const hotkey = await screen.findByLabelText('Hotkey');
        hotkey.focus();
        await userEvent.tab();
        expect(hotkey).not.toHaveFocus();
    });
});

describe('SettingsScreen files and folders', () => {
    const section = () => screen.findByRole('region', { name: 'Files and Folders' });

    const projects = 'C:\\Users\\demo\\Projects';
    const documents = 'C:\\Users\\demo\\Documents';
    const edit = (files: HTMLElement, path: string) =>
        userEvent.click(within(files).getByRole('button', { name: `Edit ${path}` }));
    const done = (files: HTMLElement, path: string) =>
        userEvent.click(within(files).getByRole('button', { name: `Done editing ${path}` }));

    it('shows rows as text, without inputs', async () => {
        renderScreen();
        const files = await section();
        const rows = within(files).getAllByRole('row');
        expect(within(rows[1]).getByText('No files')).toBeInTheDocument();
        // Folders と In search の2列
        expect(within(rows[1]).getAllByText('Yes')).toHaveLength(2);
        expect(within(rows[1]).getByText('pj')).toBeInTheDocument();
        expect(within(rows[2]).getByText('md, pdf')).toBeInTheDocument();
        expect(within(rows[2]).getByText('No')).toBeInTheDocument();
        expect(within(rows[2]).getByText('None')).toBeInTheDocument();
        expect(within(files).queryByLabelText(`Depth for ${projects}`)).not.toBeInTheDocument();
        expect(
            within(files).queryByLabelText(`Extensions for ${projects}`),
        ).not.toBeInTheDocument();
        expect(within(files).queryByLabelText(`Folders for ${projects}`)).not.toBeInTheDocument();
    });

    it('saves each row on its own, and the excluded names, only when leaving', async () => {
        jest.mocked(TauriAPI.saveSettings).mockResolvedValue();
        renderScreen();
        const files = await section();

        await edit(files, projects);
        const depth = within(files).getByLabelText(`Depth for ${projects}`);
        await userEvent.clear(depth);
        await userEvent.type(depth, '4');
        await userEvent.type(
            within(files).getByLabelText(`Extensions for ${projects}`),
            'txt, .PDF * txt',
        );
        await done(files, projects);
        await edit(files, documents);
        await userEvent.click(within(files).getByLabelText(`Folders for ${documents}`));
        await done(files, documents);
        await userEvent.click(within(files).getByRole('button', { name: 'Remove target' }));
        await userEvent.click(within(files).getByRole('button', { name: 'Add name' }));
        await userEvent.type(within(files).getByLabelText('New excluded name'), 'dist{Enter}');
        expect(TauriAPI.saveSettings).not.toHaveBeenCalled();

        await userEvent.click(screen.getByRole('button', { name: 'Back' }));
        await waitFor(() =>
            expect(TauriAPI.saveSettings).toHaveBeenCalledWith(
                expect.objectContaining({
                    localPaths: {
                        enabled: true,
                        entries: [
                            {
                                path: projects,
                                depth: 4,
                                extensions: ['txt', 'PDF', '*'],
                                includeFolders: true,
                                alias: 'pj',
                                aliasOnly: false,
                            },
                            {
                                path: documents,
                                depth: 4,
                                extensions: ['md', 'pdf'],
                                includeFolders: true,
                                aliasOnly: false,
                            },
                        ],
                        excludeNames: ['node_modules', 'dist'],
                    },
                }),
            ),
        );
    });

    it('saves the values of a row that is still being edited on Esc', async () => {
        jest.mocked(TauriAPI.saveSettings).mockResolvedValue();
        renderScreen();
        const files = await section();
        await edit(files, documents);
        const extensions = within(files).getByLabelText(`Extensions for ${documents}`);
        await userEvent.clear(extensions);
        await userEvent.type(extensions, 'txt{Escape}');
        await waitFor(() =>
            expect(TauriAPI.saveSettings).toHaveBeenCalledWith(
                expect.objectContaining({
                    localPaths: expect.objectContaining({
                        entries: [
                            expect.objectContaining({ path: projects }),
                            expect.objectContaining({ path: documents, extensions: ['txt'] }),
                        ],
                    }),
                }),
            ),
        );
    });

    it('shows the extensions as saved once the field is left, and in the row once done', async () => {
        renderScreen();
        const files = await section();
        await edit(files, documents);
        const extensions = within(files).getByLabelText(`Extensions for ${documents}`);
        expect(extensions).toHaveValue('md, pdf');
        await userEvent.clear(extensions);
        await userEvent.type(extensions, '.md,,  MD txt');
        await userEvent.tab();
        expect(extensions).toHaveValue('md, txt');
        await done(files, documents);
        expect(within(files).queryByLabelText(`Extensions for ${documents}`)).toBeNull();
        expect(within(files).getByText('md, txt')).toBeInTheDocument();
    });

    it('keeps focus on the edit button when a row opens and closes', async () => {
        renderScreen();
        const files = await section();
        await edit(files, projects);
        expect(
            within(files).getByRole('button', { name: `Done editing ${projects}` }),
        ).toHaveFocus();
        await done(files, projects);
        expect(within(files).getByRole('button', { name: `Edit ${projects}` })).toHaveFocus();
    });

    it('keeps the edited row when an earlier one is removed, and drops a removed one', async () => {
        renderScreen();
        const files = await section();
        await edit(files, documents);
        await userEvent.click(within(files).getByRole('button', { name: `Remove ${projects}` }));
        // 消した行より後ろの編集中の行は、位置が前にずれても編集中のまま
        expect(within(files).getByLabelText(`Depth for ${documents}`)).toBeInTheDocument();

        await userEvent.click(within(files).getByRole('button', { name: `Remove ${documents}` }));
        await userEvent.click(within(files).getByRole('button', { name: 'Add path' }));
        // 消した行の編集は引き継がれず、足した行だけが編集で開く
        expect(within(files).getAllByLabelText(/^Depth for /)).toHaveLength(1);
        expect(within(files).getByLabelText('Depth for new path')).toBeInTheDocument();
    });

    it('does not open another row for editing when an earlier one is removed', async () => {
        renderScreen();
        const files = await section();
        await edit(files, projects);
        await userEvent.click(within(files).getByRole('button', { name: `Remove ${projects}` }));
        expect(within(files).queryByLabelText(/^Depth for /)).not.toBeInTheDocument();
        expect(
            within(files).getByRole('button', { name: `Edit ${documents}` }),
        ).toBeInTheDocument();
    });

    it('adds an empty row in edit mode with the default rule, focused on its path', async () => {
        jest.mocked(TauriAPI.saveSettings).mockResolvedValue();
        renderScreen();
        const files = await section();
        await userEvent.click(within(files).getByRole('button', { name: 'Add path' }));
        const path = within(files).getByLabelText('Path');
        expect(path).toHaveFocus();
        expect(path).toHaveValue('');
        expect(within(files).getByLabelText('Depth for new path')).toHaveValue(3);
        expect(within(files).getByLabelText('Extensions for new path')).toHaveValue('');
        expect(within(files).getByLabelText('Folders for new path')).toBeChecked();
        expect(within(files).getByRole('button', { name: 'Add path' })).toBeDisabled();

        await userEvent.type(path, '  ~\\Notes ');
        await userEvent.click(within(files).getByRole('button', { name: /^Done editing/ }));
        expect(within(files).getByRole('button', { name: 'Edit ~\\Notes' })).toBeInTheDocument();
        expect(within(files).getByRole('button', { name: 'Add path' })).toBeEnabled();

        await userEvent.keyboard('{Escape}');
        await waitFor(() =>
            expect(TauriAPI.saveSettings).toHaveBeenCalledWith(
                expect.objectContaining({
                    localPaths: expect.objectContaining({
                        entries: [
                            expect.objectContaining({ path: projects }),
                            expect.objectContaining({ path: documents }),
                            {
                                path: '~\\Notes',
                                depth: 3,
                                extensions: [],
                                includeFolders: true,
                                aliasOnly: false,
                            },
                        ],
                    }),
                }),
            ),
        );
    });

    it('takes an alias for a folder, which can then leave the folder out of normal search', async () => {
        jest.mocked(TauriAPI.saveSettings).mockResolvedValue();
        renderScreen();
        const files = await section();
        await edit(files, documents);
        const inSearch = within(files).getByLabelText(`In search for ${documents}`);
        // 入口(Alias)がない間は、通常の検索から外せない
        expect(inSearch).toBeDisabled();
        expect(inSearch).toBeChecked();

        await userEvent.type(within(files).getByLabelText(`Alias for ${documents}`), ' doc ');
        expect(inSearch).toBeEnabled();
        await userEvent.click(inSearch);
        await done(files, documents);
        expect(within(files).queryByLabelText(`Alias for ${documents}`)).toBeNull();

        await userEvent.keyboard('{Escape}');
        await waitFor(() =>
            expect(TauriAPI.saveSettings).toHaveBeenCalledWith(
                expect.objectContaining({
                    localPaths: expect.objectContaining({
                        entries: [
                            expect.objectContaining({ path: projects }),
                            expect.objectContaining({
                                path: documents,
                                alias: 'doc',
                                aliasOnly: true,
                            }),
                        ],
                    }),
                }),
            ),
        );
    });

    it('puts a folder back in normal search when its alias is cleared', async () => {
        jest.mocked(TauriAPI.saveSettings).mockResolvedValue();
        renderScreen();
        const files = await section();
        await edit(files, projects);
        await userEvent.click(within(files).getByLabelText(`In search for ${projects}`));
        expect(within(files).getByLabelText(`In search for ${projects}`)).not.toBeChecked();
        await userEvent.clear(within(files).getByLabelText(`Alias for ${projects}`));
        expect(within(files).getByLabelText(`In search for ${projects}`)).toBeChecked();
        await done(files, projects);

        await userEvent.keyboard('{Escape}');
        await waitFor(() =>
            expect(TauriAPI.saveSettings).toHaveBeenCalledWith(
                expect.objectContaining({
                    localPaths: expect.objectContaining({
                        entries: [
                            expect.objectContaining({
                                path: projects,
                                alias: undefined,
                                aliasOnly: false,
                            }),
                            expect.objectContaining({ path: documents }),
                        ],
                    }),
                }),
            ),
        );
    });

    it('fills the path from the folder picker, also on an existing row', async () => {
        jest.mocked(TauriAPI.pickPath).mockResolvedValue('D:\\Picked');
        renderScreen();
        const files = await section();
        await edit(files, projects);
        await userEvent.click(within(files).getByRole('button', { name: 'Choose a folder' }));
        await waitFor(() => expect(within(files).getByLabelText('Path')).toHaveValue('D:\\Picked'));
        expect(TauriAPI.pickPath).toHaveBeenCalledWith(true);
        await done(files, 'D:\\Picked');
        expect(within(files).getByRole('button', { name: 'Edit D:\\Picked' })).toBeInTheDocument();
    });

    it('drops a row left blank when Done is pressed', async () => {
        renderScreen();
        const files = await section();
        await userEvent.click(within(files).getByRole('button', { name: 'Add path' }));
        await userEvent.type(within(files).getByLabelText('Path'), '   ');
        await userEvent.click(within(files).getByRole('button', { name: /^Done editing/ }));
        expect(within(files).getAllByRole('row')).toHaveLength(3);
        expect(within(files).queryByLabelText(/^Depth for /)).not.toBeInTheDocument();
        expect(within(files).getByRole('button', { name: 'Add path' })).toBeEnabled();
    });

    it('drops a blank row when another row is edited, and keeps that one in edit mode', async () => {
        renderScreen();
        const files = await section();
        await userEvent.click(within(files).getByRole('button', { name: 'Add path' }));
        await edit(files, projects);
        expect(within(files).getAllByRole('row')).toHaveLength(3);
        expect(within(files).getAllByLabelText(/^Depth for /)).toHaveLength(1);
        expect(within(files).getByLabelText(`Depth for ${projects}`)).toBeInTheDocument();
    });

    it('drops a blank row when leaving, and saves the other changes without it', async () => {
        jest.mocked(TauriAPI.saveSettings).mockResolvedValue();
        renderScreen();
        const files = await section();
        await userEvent.click(within(files).getByRole('button', { name: 'Add path' }));
        await userEvent.click(within(files).getByRole('button', { name: 'Add name' }));
        await userEvent.type(within(files).getByLabelText('New excluded name'), 'dist{Enter}');
        // 離れるまでは空の行を残す
        expect(within(files).getAllByRole('row')).toHaveLength(4);
        expect(TauriAPI.saveSettings).not.toHaveBeenCalled();

        await userEvent.click(screen.getByRole('button', { name: 'Back' }));
        await waitFor(() =>
            expect(TauriAPI.saveSettings).toHaveBeenCalledWith(
                expect.objectContaining({
                    localPaths: expect.objectContaining({
                        entries: [
                            expect.objectContaining({ path: projects }),
                            expect.objectContaining({ path: documents }),
                        ],
                        excludeNames: ['node_modules', 'target', 'dist'],
                    }),
                }),
            ),
        );
    });

    it('leaves without saving when the only change is a blank row', async () => {
        const onBack = renderScreen();
        const files = await section();
        await userEvent.click(within(files).getByRole('button', { name: 'Add path' }));
        await userEvent.keyboard('{Escape}');
        await waitFor(() => expect(onBack).toHaveBeenCalled());
        expect(TauriAPI.saveSettings).not.toHaveBeenCalled();
    });

    it('disables its fields while switched off', async () => {
        renderScreen();
        const files = await section();
        await edit(files, projects);
        await userEvent.click(
            within(files).getByRole('checkbox', { name: 'Enable Files and Folders' }),
        );
        expect(within(files).getByLabelText(`Depth for ${projects}`)).toBeDisabled();
        expect(within(files).getByRole('button', { name: `Edit ${documents}` })).toBeDisabled();
        expect(within(files).getByRole('button', { name: 'Add path' })).toBeDisabled();
        expect(within(files).getByRole('button', { name: 'Choose a folder' })).toBeDisabled();
    });
});

describe('SettingsScreen usage history', () => {
    it('saves the toggle with the other settings', async () => {
        jest.mocked(TauriAPI.saveSettings).mockResolvedValue();
        renderScreen();
        await userEvent.click(await screen.findByLabelText('Usage history'));
        await userEvent.click(screen.getByRole('button', { name: 'Back' }));
        await waitFor(() =>
            expect(TauriAPI.saveSettings).toHaveBeenCalledWith(
                expect.objectContaining({ usageHistory: false }),
            ),
        );
    });

    it('clears the history only after confirming', async () => {
        const onClearHistory = jest.fn().mockResolvedValue(undefined);
        renderScreen(jest.fn(), onClearHistory);
        await userEvent.click(await screen.findByRole('button', { name: 'Clear' }));
        expect(onClearHistory).not.toHaveBeenCalled();

        await userEvent.click(
            within(screen.getByRole('dialog')).getByRole('button', { name: 'Clear' }),
        );
        await waitFor(() => expect(onClearHistory).toHaveBeenCalledTimes(1));
        expect(await screen.findByText('History cleared')).toBeInTheDocument();
    });
});

describe('SettingsScreen open settings.json', () => {
    beforeEach(() => jest.mocked(TauriAPI.openSettingsJson).mockReset());

    it('saves edits first, then opens the file and leaves the screen', async () => {
        jest.mocked(TauriAPI.saveSettings).mockResolvedValue();
        jest.mocked(TauriAPI.openSettingsJson).mockResolvedValue();
        const onBack = renderScreen();
        await userEvent.selectOptions(await screen.findByLabelText('Theme'), 'light');
        await userEvent.click(screen.getByRole('button', { name: 'Open settings.json' }));
        await waitFor(() => expect(onBack).toHaveBeenCalled());
        expect(TauriAPI.saveSettings).toHaveBeenCalledWith(
            expect.objectContaining({ theme: 'light' }),
        );
        expect(TauriAPI.openSettingsJson).toHaveBeenCalled();
    });

    it('does not open the file while the edits cannot be saved', async () => {
        jest.mocked(TauriAPI.saveSettings).mockRejectedValue('Validation error: bad value');
        const onBack = renderScreen();
        await userEvent.selectOptions(await screen.findByLabelText('Theme'), 'light');
        await userEvent.click(screen.getByRole('button', { name: 'Open settings.json' }));
        expect(await screen.findByRole('alert')).toHaveTextContent('Could not save settings');
        expect(TauriAPI.openSettingsJson).not.toHaveBeenCalled();
        expect(onBack).not.toHaveBeenCalled();
    });

    it('stays open and says why when the file cannot be opened', async () => {
        jest.mocked(TauriAPI.openSettingsJson).mockRejectedValue(new Error('No editor'));
        const onBack = renderScreen();
        await userEvent.click(await screen.findByRole('button', { name: 'Open settings.json' }));
        expect(
            await screen.findByText(/Could not open settings.json: No editor/),
        ).toBeInTheDocument();
        expect(onBack).not.toHaveBeenCalled();
    });
});

describe('SettingsScreen startup shortcut', () => {
    const startupSwitch = () => screen.findByRole('checkbox', { name: 'Run at startup' });

    it('creates the shortcut at once, without waiting for the screen to close', async () => {
        jest.mocked(TauriAPI.setStartupShortcut).mockResolvedValue(true);
        renderScreen();
        await userEvent.click(await startupSwitch());
        expect(TauriAPI.setStartupShortcut).toHaveBeenCalledWith(true);
        expect(await screen.findByText('Scoot will run at startup')).toBeInTheDocument();
        expect(await startupSwitch()).toBeChecked();
        expect(TauriAPI.saveSettings).not.toHaveBeenCalled();
    });

    it('shows a shortcut that already exists as on, and removes it when switched off', async () => {
        jest.mocked(TauriAPI.getStartupShortcut).mockResolvedValue(true);
        jest.mocked(TauriAPI.setStartupShortcut).mockResolvedValue(false);
        renderScreen();
        const toggle = await startupSwitch();
        await waitFor(() => expect(toggle).toBeChecked());
        await userEvent.click(toggle);
        expect(TauriAPI.setStartupShortcut).toHaveBeenCalledWith(false);
        await waitFor(() => expect(toggle).not.toBeChecked());
    });

    it('says why it failed and keeps the switch as it was', async () => {
        jest.mocked(TauriAPI.setStartupShortcut).mockRejectedValue(new Error('Access is denied'));
        renderScreen();
        await userEvent.click(await startupSwitch());
        expect(await screen.findByText(/Access is denied/)).toBeInTheDocument();
        expect(await startupSwitch()).not.toBeChecked();
    });
});
