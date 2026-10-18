import { fireEvent, renderHook } from '@testing-library/react';
import { useEscapeToClose } from '../useEscapeToClose';

describe('useEscapeToClose', () => {
    it('runs on Esc', () => {
        const onEscape = jest.fn();
        renderHook(() => useEscapeToClose(onEscape));
        fireEvent.keyDown(window, { key: 'Escape' });
        expect(onEscape).toHaveBeenCalledTimes(1);
    });

    it('ignores other keys and does nothing while disabled', () => {
        const onEscape = jest.fn();
        const { rerender } = renderHook(({ on }) => useEscapeToClose(onEscape, on), {
            initialProps: { on: true },
        });
        fireEvent.keyDown(window, { key: 'Enter' });
        rerender({ on: false });
        fireEvent.keyDown(window, { key: 'Escape' });
        expect(onEscape).not.toHaveBeenCalled();
    });

    it('leaves an Esc that was already handled', () => {
        const onEscape = jest.fn();
        renderHook(() => useEscapeToClose(onEscape));
        const field = document.createElement('input');
        document.body.append(field);
        field.addEventListener('keydown', (e) => e.preventDefault());
        fireEvent.keyDown(field, { key: 'Escape' });
        field.remove();
        expect(onEscape).not.toHaveBeenCalled();
    });

    it('leaves the Esc of an open dialog to that dialog', () => {
        const onEscape = jest.fn();
        renderHook(() => useEscapeToClose(onEscape));
        const dialog = document.createElement('dialog');
        dialog.setAttribute('open', '');
        document.body.append(dialog);
        fireEvent.keyDown(dialog, { key: 'Escape' });
        dialog.remove();
        expect(onEscape).not.toHaveBeenCalled();
    });

    it('leaves an Esc that confirms an IME composition', () => {
        const onEscape = jest.fn();
        renderHook(() => useEscapeToClose(onEscape));
        fireEvent.keyDown(window, { key: 'Escape', isComposing: true });
        expect(onEscape).not.toHaveBeenCalled();
    });
});
