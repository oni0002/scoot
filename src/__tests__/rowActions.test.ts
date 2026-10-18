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

    it.each(['bookmark', 'scoot'])(
        'lets a %s result be copied, given an alias or ignored, not edited',
        (source) => {
            expect(rowActionsFor({ ...base, source })).toEqual(['copy', 'alias', 'ignore']);
        },
    );

    describe('showing the location', () => {
        const file = { ...base, category: 'file', command: 'C:/Tools/app.exe' };

        it('is offered for a file or folder, right after copy', () => {
            expect(rowActionsFor(file)).toEqual(['copy', 'reveal', 'edit', 'duplicate', 'delete']);
            expect(rowActionsFor({ ...file, source: 'application' })).toEqual([
                'copy',
                'reveal',
                'alias',
                'ignore',
            ]);
        });

        it('is not offered for a URL or a shell command', () => {
            expect(rowActionsFor({ ...base, category: 'url' })).not.toContain('reveal');
            expect(rowActionsFor({ ...base, category: 'command' })).not.toContain('reveal');
        });

        it('is not offered for a path that still has a placeholder', () => {
            expect(rowActionsFor({ ...file, command: 'C:/Logs/{$1}.txt' })).not.toContain('reveal');
        });

        it('is not offered for a typed-in path, which is not a registered command', () => {
            expect(rowActionsFor({ ...file, id: DIRECT_OPEN_ID })).toEqual(['copy']);
        });
    });

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
