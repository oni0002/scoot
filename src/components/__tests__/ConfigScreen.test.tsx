import { render, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import '@testing-library/jest-dom';
import { ConfigScreen } from '../ConfigScreen';
import { ConfigProvider, useConfigContext } from '../../context/ConfigContext';
import { NotificationProvider } from '../../context/NotificationContext';
import { TauriAPI } from '../../tauri';
import { createMockHandlers } from '../../dev/mockBackend';
import { Config } from '../../types';

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

const config = createMockHandlers().get_config({}) as Config;

const Loaded = ({
    onBack,
    onClearHistory,
}: {
    onBack: () => void;
    onClearHistory: () => Promise<void>;
}) => {
    const { config: loaded, loadConfig } = useConfigContext();
    if (!loaded) {
        void loadConfig();
        return null;
    }
    return <ConfigScreen onBack={onBack} onClearHistory={onClearHistory} />;
};

const renderScreen = (
    onBack = jest.fn(),
    onClearHistory = jest.fn().mockResolvedValue(undefined),
) => {
    render(
        <NotificationProvider>
            <ConfigProvider>
                <Loaded onBack={onBack} onClearHistory={onClearHistory} />
            </ConfigProvider>
        </NotificationProvider>,
    );
    return onBack;
};

beforeEach(() => {
    jest.mocked(TauriAPI.getConfig).mockResolvedValue(config);
    jest.mocked(TauriAPI.saveConfig).mockReset();
    jest.mocked(TauriAPI.getStartupShortcut).mockResolvedValue(false);
    jest.mocked(TauriAPI.setStartupShortcut).mockReset();
});

describe('ConfigScreen closing', () => {
    it('closes without saving when nothing changed', async () => {
        const onBack = renderScreen();
        await userEvent.click(await screen.findByRole('button', { name: 'Back' }));
        expect(onBack).toHaveBeenCalled();
        expect(TauriAPI.saveConfig).not.toHaveBeenCalled();
    });

    it('saves edits, says so, and closes', async () => {
        jest.mocked(TauriAPI.saveConfig).mockResolvedValue();
        const onBack = renderScreen();
        await userEvent.selectOptions(await screen.findByLabelText('Theme'), 'light');
        await userEvent.click(screen.getByRole('button', { name: 'Back' }));
        await waitFor(() => expect(onBack).toHaveBeenCalled());
        expect(TauriAPI.saveConfig).toHaveBeenCalledWith(
            expect.objectContaining({ theme: 'light' }),
        );
        expect(screen.getByText('Settings saved')).toBeInTheDocument();
    });

    it('stays open and shows why when saving fails', async () => {
        jest.mocked(TauriAPI.saveConfig).mockRejectedValue(
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

describe('ConfigScreen sections', () => {
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

describe('ConfigScreen usage history', () => {
    it('saves the toggle with the other settings', async () => {
        jest.mocked(TauriAPI.saveConfig).mockResolvedValue();
        renderScreen();
        await userEvent.click(await screen.findByLabelText('Usage history'));
        await userEvent.click(screen.getByRole('button', { name: 'Back' }));
        await waitFor(() =>
            expect(TauriAPI.saveConfig).toHaveBeenCalledWith(
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

describe('ConfigScreen open config.json', () => {
    beforeEach(() => jest.mocked(TauriAPI.openConfigJson).mockReset());

    it('saves edits first, then opens the file and leaves the screen', async () => {
        jest.mocked(TauriAPI.saveConfig).mockResolvedValue();
        jest.mocked(TauriAPI.openConfigJson).mockResolvedValue();
        const onBack = renderScreen();
        await userEvent.selectOptions(await screen.findByLabelText('Theme'), 'light');
        await userEvent.click(screen.getByRole('button', { name: 'Open config.json' }));
        await waitFor(() => expect(onBack).toHaveBeenCalled());
        expect(TauriAPI.saveConfig).toHaveBeenCalledWith(
            expect.objectContaining({ theme: 'light' }),
        );
        expect(TauriAPI.openConfigJson).toHaveBeenCalled();
    });

    it('does not open the file while the edits cannot be saved', async () => {
        jest.mocked(TauriAPI.saveConfig).mockRejectedValue('Validation error: bad value');
        const onBack = renderScreen();
        await userEvent.selectOptions(await screen.findByLabelText('Theme'), 'light');
        await userEvent.click(screen.getByRole('button', { name: 'Open config.json' }));
        expect(await screen.findByRole('alert')).toHaveTextContent('Could not save settings');
        expect(TauriAPI.openConfigJson).not.toHaveBeenCalled();
        expect(onBack).not.toHaveBeenCalled();
    });

    it('stays open and says why when the file cannot be opened', async () => {
        jest.mocked(TauriAPI.openConfigJson).mockRejectedValue(new Error('No editor'));
        const onBack = renderScreen();
        await userEvent.click(await screen.findByRole('button', { name: 'Open config.json' }));
        expect(
            await screen.findByText(/Could not open config.json: No editor/),
        ).toBeInTheDocument();
        expect(onBack).not.toHaveBeenCalled();
    });
});

describe('ConfigScreen startup shortcut', () => {
    const startupSwitch = () => screen.findByRole('checkbox', { name: 'Run at startup' });

    it('creates the shortcut at once, without waiting for the screen to close', async () => {
        jest.mocked(TauriAPI.setStartupShortcut).mockResolvedValue(true);
        renderScreen();
        await userEvent.click(await startupSwitch());
        expect(TauriAPI.setStartupShortcut).toHaveBeenCalledWith(true);
        expect(await screen.findByText('Scoot will run at startup')).toBeInTheDocument();
        expect(await startupSwitch()).toBeChecked();
        expect(TauriAPI.saveConfig).not.toHaveBeenCalled();
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
