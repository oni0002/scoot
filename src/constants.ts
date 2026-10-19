export const DIRECT_OPEN_ID = 'direct-open';

// DOM ids tying the search input (combobox) to the result list (listbox) and its options
export const RESULT_LIST_ID = 'search-results';
export const RESULT_HEADING_ID = 'search-results-heading';
export const resultOptionId = (index: number) => `search-result-${index}`;

export const NOTIFICATION_DURATION = {
    /** What a notification gets when its caller names no duration. */
    DEFAULT: 5000,
    UNDO: 6000,
    LONG: 8000,
    VERY_LONG: 10000,
    /** A progress toast is removed by its caller; this only keeps it from outliving a hung call. */
    PROGRESS_MAX: 120000,
};

/** How long a command may run before the launcher says it is still working. */
export const SLOW_RUN_MS = 400;

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
