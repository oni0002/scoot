import { createRef } from 'react';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import '@testing-library/jest-dom';
import { SearchBar } from '../SearchBar';

const noop = jest.fn();

const renderBar = () => {
    const inputRef = createRef<HTMLInputElement>();
    render(
        <SearchBar
            query=""
            onQueryChange={noop}
            onKeyDown={noop}
            inputRef={inputRef}
            argumentMode={null}
            onAddCommand={noop}
            onReloadCommands={noop}
            onOpenCommandsJson={noop}
            onOpenSettingsJson={noop}
            onShowAbout={noop}
            hasResults={false}
        />,
    );
    return inputRef;
};

describe('SearchBar menu', () => {
    it('exposes items as menuitems that are not tab stops', () => {
        renderBar();
        const items = screen.getAllByRole('menuitem', { hidden: true });
        expect(items.length).toBeGreaterThan(3);
        items.forEach((item) => expect(item).toHaveAttribute('tabindex', '-1'));
    });

    it('moves through the items with the arrow keys and wraps', async () => {
        renderBar();
        const trigger = screen.getByRole('button', { name: 'More actions' });
        const items = screen.getAllByRole('menuitem', { hidden: true });
        trigger.focus();
        await userEvent.keyboard('{ArrowDown}');
        expect(items[0]).toHaveFocus();
        await userEvent.keyboard('{ArrowDown}');
        expect(items[1]).toHaveFocus();
        await userEvent.keyboard('{ArrowUp}{ArrowUp}');
        expect(items[items.length - 1]).toHaveFocus();
        await userEvent.keyboard('{Home}');
        expect(items[0]).toHaveFocus();
    });

    it('Esc hands focus back to the search field', async () => {
        const inputRef = renderBar();
        screen.getByRole('button', { name: 'More actions' }).focus();
        await userEvent.keyboard('{ArrowDown}{Escape}');
        expect(inputRef.current).toHaveFocus();
    });
});
