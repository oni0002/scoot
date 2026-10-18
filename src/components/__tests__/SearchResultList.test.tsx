import { render, screen } from '@testing-library/react';
import { SearchResultList } from '../SearchResultList';
import { Command } from '../../types';

const command: Command = {
    id: 'a',
    name: 'Alpha',
    category: 'url',
    source: 'user',
    command: 'https://example.com',
    description: '',
};

const renderList = (heading?: string) =>
    render(
        <SearchResultList
            results={[{ command, score: 0, matches: [] }]}
            selectedIndex={0}
            hoveredIndex={null}
            argumentMode={null}
            query=""
            onResultClick={jest.fn()}
            onMouseEnter={jest.fn()}
            onMouseLeave={jest.fn()}
            heading={heading}
        />,
    );

describe('SearchResultList heading', () => {
    it('names the list by its heading', () => {
        renderList('Frequently used');
        expect(screen.getByRole('listbox', { name: 'Frequently used' })).toBeInTheDocument();
    });

    it('is a plain list of search results without one', () => {
        renderList();
        expect(screen.getByRole('listbox', { name: 'Search results' })).toBeInTheDocument();
        expect(screen.queryByText('Frequently used')).toBeNull();
    });
});
