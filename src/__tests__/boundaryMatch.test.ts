import { boundaryMatch } from '../boundaryMatch';

describe('boundaryMatch', () => {
    it('reads an abbreviation as word starts followed by a run', () => {
        const hit = boundaryMatch('Visual Studio Code', 'vscode');
        expect(hit?.indices).toEqual([
            [0, 0],
            [7, 7],
            [14, 17],
        ]);
        expect(hit?.words).toBe(3);
    });

    it('matches initials and prefixes of words', () => {
        expect(boundaryMatch('GitHub Desktop', 'gd')).not.toBeNull();
        expect(boundaryMatch('Visual Studio Code', 'studio')).not.toBeNull();
        expect(boundaryMatch('Visual Studio Code', 'vsc')).not.toBeNull();
    });

    it('splits words at separators and camelCase', () => {
        expect(boundaryMatch('oni0002/vscode-todomd', 'todo')).not.toBeNull();
        expect(boundaryMatch('PowerShell', 'ps')).not.toBeNull();
    });

    it('ignores spaces and case in the query', () => {
        expect(boundaryMatch('Visual Studio Code', 'VS Code')).not.toBeNull();
    });

    it('rejects letters that are not at word starts', () => {
        expect(boundaryMatch('Comet', 'vscode')).toBeNull();
        expect(boundaryMatch('Visual Studio Code', 'isual')).toBeNull();
        expect(boundaryMatch('Visual Studio Code', 'codevs')).toBeNull();
    });

    it('is null for an empty query', () => {
        expect(boundaryMatch('Visual Studio Code', '  ')).toBeNull();
    });
});
