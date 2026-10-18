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
        await userEvent.click(await screen.findByRole('button', { name: 'Add extension' }));
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

    it('lets plain Tab leave the hotkey field instead of trapping focus', async () => {
        renderScreen();
        const hotkey = await screen.findByLabelText('Hotkey');
        hotkey.focus();
        await userEvent.tab();
        expect(hotkey).not.toHaveFocus();
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
