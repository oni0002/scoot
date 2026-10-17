export const DIRECT_OPEN_ID = 'direct-open';

// DOM ids tying the search input (combobox) to the result list (listbox) and its options
export const RESULT_LIST_ID = 'search-results';
export const resultOptionId = (index: number) => `search-result-${index}`;

export const NOTIFICATION_DURATION = {
    DEFAULT: 3000,
    UNDO: 6000,
    LONG: 8000,
    VERY_LONG: 10000,
    CRITICAL: 15000,
};

export const UI_TIMEOUTS = {
    WINDOW_HIDE_DELAY: 100,
};

export const KEYBOARD_SHORTCUTS = {
    MOVE_DOWN: ['ArrowDown'],
    MOVE_UP: ['ArrowUp'],
    MOVE_DOWN_ALT: ['n'], // Ctrl+n
    MOVE_UP_ALT: ['p'], // Ctrl+p
    EXECUTE: ['Enter'],
    CANCEL: ['Escape'],
    NAVIGATE_TAB: ['Tab'],
    COPY: ['c'],
};
