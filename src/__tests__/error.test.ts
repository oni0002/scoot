import { getErrorMessage } from '../error';

describe('getErrorMessage', () => {
    it('reads the message of an Error', () => {
        expect(getErrorMessage(new Error('boom'))).toBe('boom');
    });

    it('passes strings through (Tauri rejects with plain strings)', () => {
        expect(getErrorMessage('plain failure')).toBe('plain failure');
    });

    it('falls back for unknown values', () => {
        expect(getErrorMessage({ code: 1 })).toBe('Unknown error');
    });

    it.each([
        ["Command execution error: File not found: 'C:\\x.pdf'.", "File not found: 'C:\\x.pdf'."],
        [
            "Validation error: Alias 'd' is already used by another command.",
            "Alias 'd' is already used by another command.",
        ],
        ['System error: Mutex poisoned', 'Mutex poisoned'],
        ['Not found: Command not found', 'Command not found'],
    ])('drops the backend category prefix: %s', (input, expected) => {
        expect(getErrorMessage(input)).toBe(expected);
    });

    it('keeps a message that merely mentions a category later on', () => {
        expect(getErrorMessage('Could not save: Validation error: x')).toBe(
            'Could not save: Validation error: x',
        );
    });
});
