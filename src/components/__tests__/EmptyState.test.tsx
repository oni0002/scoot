import { render, screen } from '@testing-library/react';
import '@testing-library/jest-dom';
import { NoResults, QuickGuide } from '../EmptyState';

describe('QuickGuide', () => {
    it('covers search, move and run, and nothing the hint bar already shows', () => {
        render(<QuickGuide />);
        for (const label of ['Search', 'Move', 'Run']) {
            expect(screen.getByText(label)).toBeInTheDocument();
        }
        expect(screen.queryByText('Copy')).not.toBeInTheDocument();
        expect(screen.queryByText('Arguments')).not.toBeInTheDocument();
    });
});

describe('QuickGuide alias example', () => {
    it('shows a real alias when one takes arguments', () => {
        render(<QuickGuide aliasExample="d" />);
        expect(screen.getByText('d text')).toBeInTheDocument();
    });

    it('does not mention aliases when none exist', () => {
        render(<QuickGuide />);
        expect(screen.queryByText(/alias/)).not.toBeInTheDocument();
    });
});

describe('NoResults', () => {
    it('names the query and a way out', () => {
        render(<NoResults query="zzz" />);
        expect(screen.getByText('No commands found for "zzz"')).toBeInTheDocument();
        expect(screen.getByText(/clears the search/)).toBeInTheDocument();
    });
});
