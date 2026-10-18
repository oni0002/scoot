import React, { useCallback, useEffect, useRef, useState } from 'react';
import { listenTo } from '../listen';
import { TauriAPI } from '../tauri';

/**
 * Hotkey capture for the settings field. Alt+<key> combos are intercepted at the Win32 level
 * (WM_SYSCOMMAND SC_KEYMENU subclass) and arrive as a `hotkey-captured` event; other combos
 * (Ctrl+A, Shift+F1, ...) are read from the DOM key event.
 */
export const useHotkeyCapture = (onCapture: (hotkey: string) => void) => {
    const [capturing, setCapturing] = useState(false);
    const inputRef = useRef<HTMLInputElement>(null);

    const onCaptureRef = useRef(onCapture);
    useEffect(() => {
        onCaptureRef.current = onCapture;
    }, [onCapture]);

    const finish = useCallback((hotkey?: string) => {
        setCapturing(false);
        inputRef.current?.blur();
        if (hotkey) onCaptureRef.current(hotkey);
    }, []);

    useEffect(() => {
        if (!capturing) return;
        void TauriAPI.startHotkeyCapture();
        const stopListening = listenTo<string>('hotkey-captured', finish);
        return () => {
            void TauriAPI.stopHotkeyCapture();
            stopListening();
        };
    }, [capturing, finish]);

    const onKeyDown = useCallback(
        (e: React.KeyboardEvent<HTMLInputElement>) => {
            // A plain Tab is never a hotkey (it needs a modifier), so let it move focus on
            if (e.key === 'Tab' && !e.ctrlKey && !e.altKey && !e.shiftKey && !e.metaKey) return;

            e.preventDefault();
            e.stopPropagation();

            if (e.key === 'Escape') {
                finish();
                return;
            }

            const modifiers: string[] = [];
            if (e.ctrlKey) modifiers.push('Ctrl');
            if (e.altKey) modifiers.push('Alt');
            if (e.shiftKey) modifiers.push('Shift');
            if (e.metaKey) modifiers.push('Super');

            const ignoredKeys = ['Control', 'Alt', 'Shift', 'Meta', 'Super'];
            if (ignoredKeys.includes(e.key)) return;
            if (modifiers.length === 0) return;

            const keyMap: Record<string, string> = { ' ': 'Space', Enter: 'Enter', Tab: 'Tab' };
            const key = keyMap[e.key] ?? e.key.toUpperCase();
            finish([...modifiers, key].join('+'));
        },
        [finish],
    );

    return {
        capturing,
        inputProps: {
            ref: inputRef,
            onFocus: () => setCapturing(true),
            onBlur: () => setCapturing(false),
            onKeyDown,
        },
    };
};

export type HotkeyCapture = ReturnType<typeof useHotkeyCapture>;
