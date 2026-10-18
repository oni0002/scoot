import React from 'react';
import { renderHook } from '@testing-library/react';
import { useKeyboardNavigation } from '../useKeyboardNavigation';

const setup = () => {
    const props = {
        moveSelection: jest.fn(),
        executeCommand: jest.fn(),
        copySelectedCommand: jest.fn(),
        onRowAction: jest.fn(),
        resetState: jest.fn(),
        query: 'd',
        setQuery: jest.fn(),
        searchMode: { mode: 'idle' } as const,
        setSearchMode: jest.fn(),
        inputRef: { current: document.createElement('input') },
    };
    const { result } = renderHook(() => useKeyboardNavigation(props));
    const press = (key: string, init: Partial<React.KeyboardEvent> = {}) => {
        const event = { key, preventDefault: jest.fn(), ...init } as unknown as React.KeyboardEvent;
        result.current.handleKeyDown(event);
        return event;
    };
    return { props, press };
};

describe('row action shortcuts', () => {
    it.each([
        ['e', 'edit'],
        ['d', 'delete'],
        ['l', 'alias'],
        ['i', 'ignore'],
    ])('Ctrl+%s runs %s on the selected row', (key, action) => {
        const { props, press } = setup();
        const event = press(key, { ctrlKey: true });
        expect(props.onRowAction).toHaveBeenCalledWith(action);
        expect(event.preventDefault).toHaveBeenCalled();
    });

    it('treats the key case-insensitively (Caps Lock, Shift)', () => {
        const { props, press } = setup();
        press('E', { ctrlKey: true });
        expect(props.onRowAction).toHaveBeenCalledWith('edit');
    });

    it('ignores the letters without Ctrl so typing is unaffected', () => {
        const { props, press } = setup();
        const event = press('e');
        expect(props.onRowAction).not.toHaveBeenCalled();
        expect(event.preventDefault).not.toHaveBeenCalled();
    });

    it('ignores Ctrl+Alt combinations (AltGr on some layouts)', () => {
        const { props, press } = setup();
        press('e', { ctrlKey: true, altKey: true });
        expect(props.onRowAction).not.toHaveBeenCalled();
    });

    it('still copies on Ctrl+C when no text is selected', () => {
        const { props, press } = setup();
        press('c', { ctrlKey: true });
        expect(props.copySelectedCommand).toHaveBeenCalled();
        expect(props.onRowAction).not.toHaveBeenCalled();
    });
});
