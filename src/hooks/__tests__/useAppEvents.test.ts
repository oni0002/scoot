import { renderHook, waitFor } from '@testing-library/react';
import { useAppEvents } from '../useAppEvents';
import { TauriAPI } from '../../tauri';

jest.mock('../../tauri');

const setup = () => {
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
    return props;
};

describe('useAppEvents hotkey warning', () => {
    it('shows why the Hotkey could not be registered at startup', async () => {
        jest.mocked(TauriAPI.getShortcutWarning).mockResolvedValue(
            'Alt+Space is not available. Pick another Hotkey in Settings.',
        );
        const props = setup();
        await waitFor(() =>
            expect(props.showWarning).toHaveBeenCalledWith(
                'Alt+Space is not available. Pick another Hotkey in Settings.',
            ),
        );
    });

    it('says nothing when the Hotkey is registered', async () => {
        jest.mocked(TauriAPI.getShortcutWarning).mockResolvedValue(null);
        const props = setup();
        await waitFor(() => expect(props.loadCommands).toHaveBeenCalled());
        expect(TauriAPI.getShortcutWarning).toHaveBeenCalled();
        expect(props.showWarning).not.toHaveBeenCalled();
    });

    it('carries on when the status cannot be read', async () => {
        jest.spyOn(console, 'warn').mockImplementation(() => {});
        jest.mocked(TauriAPI.getShortcutWarning).mockRejectedValue(new Error('no'));
        const props = setup();
        await waitFor(() => expect(props.loadCommands).toHaveBeenCalled());
        expect(props.showWarning).not.toHaveBeenCalled();
    });
});
