import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import '@testing-library/jest-dom';
import { CommandForm } from '../CommandForm';

jest.mock('@tauri-apps/plugin-dialog', () => ({ open: jest.fn() }));
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

describe('CommandForm leaving', () => {
    it('closes at once when nothing was typed', async () => {
        const onCancel = jest.fn();
        render(<CommandForm onSave={jest.fn()} onCancel={onCancel} />);
        await userEvent.keyboard('{Escape}');
        expect(onCancel).toHaveBeenCalledTimes(1);
    });

    it('asks before throwing away edits, and Keep editing stays', async () => {
        const onCancel = jest.fn();
        render(<CommandForm onSave={jest.fn()} onCancel={onCancel} />);
        await userEvent.type(screen.getByLabelText(/name/i), 'x');
        await userEvent.keyboard('{Escape}');
        expect(screen.getByRole('dialog')).toHaveTextContent('Discard changes?');
        expect(onCancel).not.toHaveBeenCalled();

        await userEvent.click(screen.getByRole('button', { name: 'Keep editing' }));
        expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
        expect(onCancel).not.toHaveBeenCalled();
    });

    it('Discard leaves the form', async () => {
        const onCancel = jest.fn();
        render(<CommandForm onSave={jest.fn()} onCancel={onCancel} />);
        await userEvent.type(screen.getByLabelText(/name/i), 'x');
        await userEvent.click(screen.getByRole('button', { name: 'Cancel' }));
        await userEvent.click(screen.getByRole('button', { name: 'Discard' }));
        expect(onCancel).toHaveBeenCalledTimes(1);
    });

    it('does not count the initial values of an edited command as changes', async () => {
        const onCancel = jest.fn();
        render(
            <CommandForm
                command={{
                    id: 'u1',
                    name: 'A',
                    category: 'url',
                    source: 'user',
                    command: 'https://a.dev',
                    description: 'd',
                }}
                onSave={jest.fn()}
                onCancel={onCancel}
            />,
        );
        await userEvent.keyboard('{Escape}');
        expect(onCancel).toHaveBeenCalledTimes(1);
    });
});

describe('CommandForm validation copy', () => {
    it('names the field that is missing for the category', async () => {
        render(<CommandForm onSave={jest.fn()} onCancel={jest.fn()} />);
        await userEvent.click(screen.getByRole('button', { name: 'Add' }));
        expect(screen.getByText('URL is required')).toBeInTheDocument();
        expect(screen.queryByText('Command is required')).not.toBeInTheDocument();
    });

    it('says why the alias is disabled', () => {
        render(<CommandForm onSave={jest.fn()} onCancel={jest.fn()} />);
        expect(screen.getByLabelText(/alias/i)).toBeDisabled();
        expect(screen.getByText(/Enabled when the command has/)).toBeInTheDocument();
    });
});

describe('CommandForm duplicating', () => {
    const template = {
        id: 'c1',
        name: 'git status',
        category: 'command',
        source: 'user',
        command: 'git {$1}',
        description: 'Git',
        alias: 'g',
    };

    it('starts from the original but saves a new command without its id or alias', async () => {
        const onSave = jest.fn();
        render(<CommandForm template={template} onSave={onSave} onCancel={jest.fn()} />);
        expect(screen.getByLabelText(/name/i)).toHaveValue('git status');
        expect(screen.getByLabelText(/alias/i)).toHaveValue('');

        await userEvent.type(screen.getByLabelText(/alias/i), 'gs');
        await userEvent.click(screen.getByRole('button', { name: 'Add' }));
        expect(onSave).toHaveBeenCalledWith(
            expect.objectContaining({ id: '', name: 'git status', alias: 'gs' }),
        );
    });
});
