import { renderHook, waitFor } from '@testing-library/react';
import { listen } from '@tauri-apps/api/event';
import { useAppEvents } from '../useAppEvents';
import { TauriAPI } from '../../tauri';

jest.mock('../../tauri');

const WARNING = 'Alt+Space is not available. Pick another Hotkey in Settings.';

type Handler = (event: { payload?: unknown }) => void;

const setup = async () => {
    const handlers: Record<string, Handler> = {};
    jest.mocked(listen).mockImplementation(async (event, callback) => {
        handlers[event] = callback as Handler;
        return () => {};
    });
    const props = {
        handleAddCommand: jest.fn(),
        handleOpenConfig: jest.fn(),
        loadCommands: jest.fn(),
        loadConfig: jest.fn(),
        showWarning: jest.fn(),
        showError: jest.fn(),
        setError: jest.fn(),
    };
    renderHook(() => useAppEvents(props));
    await waitFor(() => expect(props.loadCommands).toHaveBeenCalled()); // subscribed
    const windowShown = async () => {
        handlers['window-shown']({});
        await Promise.resolve();
        await Promise.resolve();
    };
    return { props, handlers, windowShown };
};

describe('useAppEvents hotkey warning', () => {
    it('does not show anything at startup, while the window is hidden', async () => {
        jest.mocked(TauriAPI.getShortcutWarning).mockResolvedValue(WARNING);
        const { props } = await setup();
        expect(TauriAPI.getShortcutWarning).not.toHaveBeenCalled();
        expect(props.showWarning).not.toHaveBeenCalled();
    });

    it('shows why the Hotkey could not be registered when the window is shown', async () => {
        jest.mocked(TauriAPI.getShortcutWarning).mockResolvedValue(WARNING);
        const { props, windowShown } = await setup();
        await windowShown();
        expect(props.showWarning).toHaveBeenCalledWith(WARNING, expect.any(Number));
    });

    it('does not repeat the same warning each time the window opens', async () => {
        jest.mocked(TauriAPI.getShortcutWarning).mockResolvedValue(WARNING);
        const { props, windowShown } = await setup();
        await windowShown();
        await windowShown();
        expect(props.showWarning).toHaveBeenCalledTimes(1);
    });

    it('says nothing when the Hotkey is registered', async () => {
        jest.mocked(TauriAPI.getShortcutWarning).mockResolvedValue(null);
        const { props, windowShown } = await setup();
        await windowShown();
        expect(props.showWarning).not.toHaveBeenCalled();
    });

    it('carries on when the status cannot be read', async () => {
        jest.spyOn(console, 'warn').mockImplementation(() => {});
        jest.mocked(TauriAPI.getShortcutWarning).mockRejectedValue(new Error('no'));
        const { props, windowShown } = await setup();
        await windowShown();
        expect(props.showWarning).not.toHaveBeenCalled();
    });

    it('shows a warning that arrives as an event every time, and not again on the next opening', async () => {
        jest.mocked(TauriAPI.getShortcutWarning).mockResolvedValue(WARNING);
        const { props, handlers, windowShown } = await setup();
        handlers['shortcut-warning']({ payload: WARNING });
        handlers['shortcut-warning']({ payload: WARNING }); // saved twice: two notices
        expect(props.showWarning).toHaveBeenCalledTimes(2);
        await windowShown(); // already told
        expect(props.showWarning).toHaveBeenCalledTimes(2);
    });
});
