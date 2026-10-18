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

    it('keeps the alias optional unless the command takes arguments', async () => {
        render(<CommandForm onSave={jest.fn()} onCancel={jest.fn()} />);
        expect(screen.getByLabelText(/alias/i)).toBeEnabled();
        expect(screen.getByText('Optional short name')).toBeInTheDocument();

        await userEvent.click(screen.getByLabelText(/url/i));
        await userEvent.paste('https://a.dev/?q={$1}');
        expect(screen.getByLabelText(/alias \*/i)).toBeInTheDocument();
        await userEvent.type(screen.getByLabelText(/name/i), 'A');
        await userEvent.click(screen.getByRole('button', { name: 'Add' }));
        expect(screen.getByText('Required for args')).toBeInTheDocument();
    });

    it('saves an alias on a command without arguments and keeps it when the command changes', async () => {
        const onSave = jest.fn();
        render(<CommandForm onSave={onSave} onCancel={jest.fn()} />);
        await userEvent.type(screen.getByLabelText(/name/i), 'Docs');
        await userEvent.type(screen.getByLabelText(/url/i), 'https://a.dev');
        await userEvent.type(screen.getByLabelText(/alias/i), 'docs');
        await userEvent.type(screen.getByLabelText(/url/i), '/x');
        await userEvent.click(screen.getByRole('button', { name: 'Add' }));
        expect(onSave).toHaveBeenCalledWith(
            expect.objectContaining({ command: 'https://a.dev/x', alias: 'docs' }),
        );
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
        expect(screen.getByRole('heading', { name: 'Duplicate command' })).toBeInTheDocument();
        expect(screen.getByLabelText(/name/i)).toHaveValue('git status copy');
        expect(screen.getByLabelText(/alias/i)).toHaveValue('');

        await userEvent.type(screen.getByLabelText(/alias/i), 'gs');
        await userEvent.click(screen.getByRole('button', { name: 'Add' }));
        expect(onSave).toHaveBeenCalledWith(
            expect.objectContaining({ id: '', name: 'git status copy', alias: 'gs' }),
        );
    });
});
