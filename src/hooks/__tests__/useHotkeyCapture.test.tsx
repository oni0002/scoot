import { act, render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { useHotkeyCapture } from '../useHotkeyCapture';
import { TauriAPI } from '../../tauri';

jest.mock('../../tauri');

const Field = ({ onCapture }: { onCapture: (hotkey: string) => void }) => {
    const { capturing, inputProps } = useHotkeyCapture(onCapture);
    return (
        <input
            aria-label="hotkey"
            readOnly
            value={capturing ? 'capturing' : 'idle'}
            {...inputProps}
        />
    );
};

describe('useHotkeyCapture', () => {
    it('captures a modifier combination and stops capturing', async () => {
        const onCapture = jest.fn();
        render(<Field onCapture={onCapture} />);
        const input = screen.getByLabelText('hotkey');

        await act(async () => input.focus());
        expect(input).toHaveValue('capturing');
        expect(TauriAPI.startHotkeyCapture).toHaveBeenCalled();

        await userEvent.keyboard('{Control>}{Shift>}k{/Shift}{/Control}');
        expect(onCapture).toHaveBeenCalledWith('Ctrl+Shift+K');
        expect(input).toHaveValue('idle');
    });

    it('ignores a key without a modifier', async () => {
        const onCapture = jest.fn();
        render(<Field onCapture={onCapture} />);
        await act(async () => screen.getByLabelText('hotkey').focus());

        await userEvent.keyboard('k');
        expect(onCapture).not.toHaveBeenCalled();
    });

    it('cancels on Escape without capturing', async () => {
        const onCapture = jest.fn();
        render(<Field onCapture={onCapture} />);
        const input = screen.getByLabelText('hotkey');
        await act(async () => input.focus());

        await userEvent.keyboard('{Escape}');
        expect(onCapture).not.toHaveBeenCalled();
        expect(input).toHaveValue('idle');
    });
});
