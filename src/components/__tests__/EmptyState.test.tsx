import { render, screen } from '@testing-library/react';
import '@testing-library/jest-dom';
import { NoResults, QuickGuide } from '../EmptyState';

describe('QuickGuide', () => {
    it('shows the configured hotkey, not a fixed one', () => {
        render(<QuickGuide hotkey="Ctrl+Shift+K" />);
        const row = screen.getByText('Show / hide').nextElementSibling as HTMLElement;
        expect(row).toHaveTextContent('Ctrl+Shift+K');
        expect(screen.queryByText('Alt')).not.toBeInTheDocument();
    });

    it('covers arguments, direct open and copy', () => {
        render(<QuickGuide hotkey="Alt+Space" />);
        expect(screen.getByText('Arguments')).toBeInTheDocument();
        expect(screen.getByText('Open')).toBeInTheDocument();
        expect(screen.getByText('Copy')).toBeInTheDocument();
    });
});

describe('NoResults', () => {
    it('names the query and a way out', () => {
        render(<NoResults query="zzz" />);
        expect(screen.getByText('No commands found for "zzz"')).toBeInTheDocument();
        expect(screen.getByText(/clears the search/)).toBeInTheDocument();
    });
});
