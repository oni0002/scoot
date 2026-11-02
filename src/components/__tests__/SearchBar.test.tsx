import { createRef } from 'react';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import '@testing-library/jest-dom';
import { LuFolderSearch } from 'react-icons/lu';
import { SearchBar } from '../SearchBar';

const noop = jest.fn();

const barProps = (inputRef = createRef<HTMLInputElement>()) => ({
    query: '',
    onQueryChange: noop,
    onKeyDown: noop,
    inputRef,
    argumentMode: null,
    scoped: false,
    onAddCommand: noop,
    onReloadCommands: noop,
    onOpenCommandsJson: noop,
    onOpenSettingsJson: noop,
    onShowAbout: noop,
    hasResults: false,
});

const renderBar = () => {
    const inputRef = createRef<HTMLInputElement>();
    render(<SearchBar {...barProps(inputRef)} />);
    return inputRef;
};

describe('SearchBar menu', () => {
    it('swaps the leading icon for a folder search icon when scoped', () => {
        const leadingIcon = (scoped: boolean) => {
            const { container, unmount } = render(<SearchBar {...barProps()} scoped={scoped} />);
            const html = container.querySelector('label svg')?.outerHTML;
            unmount();
            return html;
        };
        const folder = render(<LuFolderSearch />);
        const expected = folder.container.innerHTML;
        folder.unmount();
        expect(leadingIcon(true)).toBe(expected);
        expect(leadingIcon(false)).not.toBe(expected);
    });

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
