import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import '@testing-library/jest-dom';
import { ArgumentPanel } from '../ArgumentPanel';
import { Command } from '../../types';

const search: Command = {
    id: 'u1',
    name: 'Duckduckgo search',
    category: 'url',
    source: 'user',
    command: 'https://duckduckgo.com/search?q={$1}',
    description: 'ddg search',
    alias: 'd',
};

describe('ArgumentPanel', () => {
    it('identifies the command and shows the alias', () => {
        render(<ArgumentPanel command={search} args={[]} />);
        expect(screen.getByText('Duckduckgo search')).toBeInTheDocument();
        expect(screen.getByText('d')).toBeInTheDocument();
    });

    it('says what is still missing before anything is typed', () => {
        render(<ArgumentPanel command={search} args={[]} />);
        expect(screen.getByRole('status')).toHaveTextContent('Waiting for {$1}');
    });

    it('shows the URL with the typed argument filled in and encoded', () => {
        render(<ArgumentPanel command={search} args={['c++']} />);
        expect(screen.getByText('c%2B%2B')).toBeInTheDocument();
        expect(screen.getByRole('status')).toBeEmptyDOMElement();
    });

    it('warns about an argument the command would drop', () => {
        render(<ArgumentPanel command={search} args={['react', 'hooks']} />);
        expect(screen.getByRole('status')).toHaveTextContent(
            '"hooks" is not used: this command only takes {$1}.',
        );
    });

    it('does not warn when {$*} takes every argument', () => {
        const all: Command = { ...search, command: 'https://x.dev/?q={$*}' };
        render(<ArgumentPanel command={all} args={['a', 'b', 'c']} />);
        expect(screen.getByRole('status')).toBeEmptyDOMElement();
    });

    it('describes a shell command as running rather than opening', () => {
        const shell: Command = { ...search, category: 'command', command: 'echo {$1}' };
        render(<ArgumentPanel command={shell} args={['hi']} />);
        expect(screen.getByText('Runs')).toBeInTheDocument();
    });

    it('offers a mouse copy button that calls onCopy', async () => {
        const onCopy = jest.fn();
        render(<ArgumentPanel command={search} args={['react']} onCopy={onCopy} />);
        // aria-hidden like the row buttons: the keyboard path is Ctrl+C
        await userEvent.click(screen.getByRole('button', { name: 'Copy result', hidden: true }));
        expect(onCopy).toHaveBeenCalledTimes(1);
    });
});
