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
