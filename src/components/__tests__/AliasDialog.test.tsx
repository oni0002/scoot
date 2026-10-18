import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import '@testing-library/jest-dom';
import { AliasDialog } from '../AliasDialog';
import { Command } from '../../types';

jest.mock('../../tauri');

// jsdom does not implement <dialog> modals
beforeAll(() => {
    HTMLDialogElement.prototype.showModal = function () {
        this.setAttribute('open', '');
    };
    HTMLDialogElement.prototype.close = function () {
        if (!this.hasAttribute('open')) return;
        this.removeAttribute('open');
        this.dispatchEvent(new Event('close'));
    };
});

const app: Command = {
    id: 'ext-1',
    name: 'Visual Studio Code',
    category: 'file',
    source: 'application',
    command: 'C:\\Apps\\Code.lnk',
    description: '',
};

describe('AliasDialog', () => {
    it('renders nothing without a command', () => {
        render(<AliasDialog onSubmit={jest.fn()} onCancel={jest.fn()} />);
        expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
    });

    it('saves the trimmed alias', async () => {
        const onSubmit = jest.fn().mockResolvedValue(undefined);
        render(<AliasDialog command={app} onSubmit={onSubmit} onCancel={jest.fn()} />);
        await userEvent.type(screen.getByLabelText(/short name/i), ' vscode ');
        await userEvent.click(screen.getByRole('button', { name: 'Save' }));
        expect(onSubmit).toHaveBeenCalledWith('vscode');
    });

    it('treats an emptied field as removing the alias', async () => {
        const onSubmit = jest.fn().mockResolvedValue(undefined);
        render(
            <AliasDialog
                command={{ ...app, alias: 'vscode' }}
                onSubmit={onSubmit}
                onCancel={jest.fn()}
            />,
        );
        await userEvent.clear(screen.getByLabelText(/short name/i));
        await userEvent.click(screen.getByRole('button', { name: 'Save' }));
        expect(onSubmit).toHaveBeenCalledWith(null);
    });

    it('offers Remove only when there is an alias to remove', async () => {
        const { rerender } = render(
            <AliasDialog command={app} onSubmit={jest.fn()} onCancel={jest.fn()} />,
        );
        expect(screen.queryByRole('button', { name: 'Remove' })).not.toBeInTheDocument();

        rerender(
            <AliasDialog
                command={{ ...app, id: 'ext-2', alias: 'vscode' }}
                onSubmit={jest.fn().mockResolvedValue(undefined)}
                onCancel={jest.fn()}
            />,
        );
        expect(screen.getByRole('button', { name: 'Remove' })).toBeInTheDocument();
    });

    it('stays open and shows why when the alias is rejected', async () => {
        const onSubmit = jest
            .fn()
            .mockRejectedValue(new Error("Alias 'x' is already used by another command."));
        const onCancel = jest.fn();
        render(<AliasDialog command={app} onSubmit={onSubmit} onCancel={onCancel} />);
        await userEvent.type(screen.getByLabelText(/short name/i), 'x');
        await userEvent.click(screen.getByRole('button', { name: 'Save' }));
        expect(await screen.findByRole('alert')).toHaveTextContent('already used');
        expect(onCancel).not.toHaveBeenCalled();
    });

    it('cancels with the Cancel button', async () => {
        const onCancel = jest.fn();
        render(<AliasDialog command={app} onSubmit={jest.fn()} onCancel={onCancel} />);
        await userEvent.click(screen.getByRole('button', { name: 'Cancel' }));
        expect(onCancel).toHaveBeenCalledTimes(1);
    });
});
