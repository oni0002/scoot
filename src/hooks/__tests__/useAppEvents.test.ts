import { renderHook, waitFor } from '@testing-library/react';
import { listen } from '@tauri-apps/api/event';
import { useAppEvents } from '../useAppEvents';
import { TauriAPI } from '../../tauri';
import { Notice } from '../../types';

jest.mock('../../tauri');

const HOTKEY: Notice = {
    source: 'hotkey',
    level: 'warning',
    message: 'Alt+Space is not available. Pick another Hotkey in Settings.',
};
const SETTINGS: Notice = {
    source: 'settings',
    level: 'error',
    message:
        'System error: App settings file validation failed: maxResults must be between 1 and 100',
};

type Handler = (event: { payload?: unknown }) => void;

const setup = async () => {
    const handlers: Record<string, Handler> = {};
    jest.mocked(listen).mockImplementation(async (event, callback) => {
        handlers[event] = callback as Handler;
        return () => {};
    });
    const props = {
        handleAddCommand: jest.fn(),
        handleOpenSettings: jest.fn(),
        loadCommands: jest.fn(),
        loadSettings: jest.fn(),
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

describe('useAppEvents notices', () => {
    beforeEach(() => jest.mocked(TauriAPI.getNotices).mockResolvedValue([]));

    it('shows nothing at startup, while the window is hidden', async () => {
        jest.mocked(TauriAPI.getNotices).mockResolvedValue([HOTKEY, SETTINGS]);
        const { props } = await setup();
        expect(TauriAPI.getNotices).not.toHaveBeenCalled();
        expect(props.showWarning).not.toHaveBeenCalled();
        expect(props.showError).not.toHaveBeenCalled();
    });

    it('shows what waited, by level, when the window is shown', async () => {
        jest.mocked(TauriAPI.getNotices).mockResolvedValue([SETTINGS, HOTKEY]);
        const { props, windowShown } = await setup();
        await windowShown();
        expect(props.showWarning).toHaveBeenCalledWith(HOTKEY.message, expect.any(Number));
        // without the "System error: " category the backend puts in front
        expect(props.showError).toHaveBeenCalledWith(
            'App settings file validation failed: maxResults must be between 1 and 100',
            expect.any(Number),
        );
    });

    it('does not repeat a notice each time the window opens', async () => {
        jest.mocked(TauriAPI.getNotices).mockResolvedValue([HOTKEY]);
        const { props, windowShown } = await setup();
        await windowShown();
        await windowShown();
        expect(props.showWarning).toHaveBeenCalledTimes(1);
    });

    it('tells again when the same problem comes back after it was gone', async () => {
        const { props, windowShown } = await setup();
        jest.mocked(TauriAPI.getNotices).mockResolvedValue([HOTKEY]);
        await windowShown();
        jest.mocked(TauriAPI.getNotices).mockResolvedValue([]); // fixed
        await windowShown();
        jest.mocked(TauriAPI.getNotices).mockResolvedValue([HOTKEY]); // broken again
        await windowShown();
        expect(props.showWarning).toHaveBeenCalledTimes(2);
    });

    it('tells a changed message from the same source again', async () => {
        const { props, windowShown } = await setup();
        jest.mocked(TauriAPI.getNotices).mockResolvedValue([SETTINGS]);
        await windowShown();
        jest.mocked(TauriAPI.getNotices).mockResolvedValue([{ ...SETTINGS, message: 'another' }]);
        await windowShown();
        expect(props.showError).toHaveBeenCalledTimes(2);
    });

    it('shows a notice that arrives while the window is showing, every time', async () => {
        const { props, handlers, windowShown } = await setup();
        handlers['notice']({ payload: HOTKEY });
        handlers['notice']({ payload: HOTKEY }); // saved twice: two notices
        expect(props.showWarning).toHaveBeenCalledTimes(2);
        jest.mocked(TauriAPI.getNotices).mockResolvedValue([HOTKEY]);
        await windowShown(); // already told
        expect(props.showWarning).toHaveBeenCalledTimes(2);
    });

    it('says nothing when there is nothing, and carries on when the notices cannot be read', async () => {
        jest.spyOn(console, 'warn').mockImplementation(() => {});
        const { props, windowShown } = await setup();
        await windowShown();
        jest.mocked(TauriAPI.getNotices).mockRejectedValue(new Error('no'));
        await windowShown();
        expect(props.showWarning).not.toHaveBeenCalled();
        expect(props.showError).not.toHaveBeenCalled();
    });
});
