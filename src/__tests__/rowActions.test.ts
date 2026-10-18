import { DIRECT_OPEN_ID } from '../constants';
import { ROW_ACTION_BY_KEY, ROW_ACTIONS, rowActionsFor } from '../rowActions';
import { Command } from '../types';

const base: Command = {
    id: 'c1',
    name: 'Example',
    category: 'url',
    source: 'user',
    command: 'https://example.com',
    description: '',
};

describe('rowActionsFor', () => {
    it('lets a custom command be copied, edited, duplicated and deleted', () => {
        expect(rowActionsFor(base)).toEqual(['copy', 'edit', 'duplicate', 'delete']);
    });

    it.each(['bookmark', 'application', 'scoot'])(
        'lets a %s result be copied, given an alias or ignored, not edited',
        (source) => {
            expect(rowActionsFor({ ...base, source })).toEqual(['copy', 'alias', 'ignore']);
        },
    );

    it('only lets a typed-in URL or path be copied', () => {
        expect(rowActionsFor({ ...base, id: DIRECT_OPEN_ID })).toEqual(['copy']);
    });
});

describe('keyboard accelerators', () => {
    it('map to actions that exist and advertise the same shortcut', () => {
        for (const [key, action] of Object.entries(ROW_ACTION_BY_KEY)) {
            expect(ROW_ACTIONS[action].shortcut).toBe(`Ctrl+${key.toUpperCase()}`);
        }
    });
});
