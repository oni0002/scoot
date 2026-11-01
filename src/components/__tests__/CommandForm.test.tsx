import { act, render, screen } from '@testing-library/react';
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

    it.each([
        ['url', 'URL *', 'https://example.com', 'URL is required'],
        ['file', 'Path *', 'C:\\path\\to\\file.txt', 'Path is required'],
        ['command', 'Command *', 'Command line', 'Command is required'],
    ])(
        'words the %s category as label, placeholder and error',
        async (category, label, placeholder, error) => {
            render(<CommandForm onSave={jest.fn()} onCancel={jest.fn()} />);
            await userEvent.selectOptions(screen.getByLabelText(/category/i), category);
            expect(screen.getByLabelText(label)).toHaveAttribute('placeholder', placeholder);

            await userEvent.click(screen.getByRole('button', { name: 'Add' }));
            expect(screen.getByText(error)).toBeInTheDocument();
        },
    );

    it('keeps the alias optional unless the command takes arguments', async () => {
        render(<CommandForm onSave={jest.fn()} onCancel={jest.fn()} />);
        expect(screen.getByLabelText(/alias/i)).toBeEnabled();
        expect(screen.getByText('Optional short name')).toBeInTheDocument();

        await userEvent.click(screen.getByLabelText(/url/i));
        await userEvent.paste('https://a.dev/?q={$1}');
        expect(screen.getByLabelText(/alias \*/i)).toBeInTheDocument();
        expect(screen.getByText('Type "g react" to pass react as {$1}.')).toBeInTheDocument();
        await userEvent.type(screen.getByLabelText(/alias/i), 'gs');
        expect(screen.getByText('Type "gs react" to pass react as {$1}.')).toBeInTheDocument();
        await userEvent.clear(screen.getByLabelText(/alias/i));
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

describe('CommandForm saving', () => {
    it('ignores a second submit while the first save is still pending', async () => {
        let finish: () => void = () => {};
        const onSave = jest.fn(() => new Promise<void>((resolve) => (finish = resolve)));
        render(<CommandForm onSave={onSave} onCancel={jest.fn()} />);
        await userEvent.type(screen.getByLabelText(/name/i), 'Docs');
        await userEvent.click(screen.getByLabelText(/url/i));
        await userEvent.paste('https://docs.dev');

        await userEvent.keyboard('{Enter}{Enter}');
        expect(onSave).toHaveBeenCalledTimes(1);
        expect(screen.getByRole('button', { name: 'Add' })).toBeDisabled();

        await act(async () => finish());
        expect(screen.getByRole('button', { name: 'Add' })).toBeEnabled();
    });
});

describe('CommandForm open with', () => {
    it.each([
        ['url', true],
        ['file', true],
        ['command', false],
    ])('shows the program and its arguments for %s: %s', async (category, shown) => {
        render(<CommandForm onSave={jest.fn()} onCancel={jest.fn()} />);
        await userEvent.selectOptions(screen.getByLabelText(/category/i), category);
        expect(!!screen.queryByLabelText('Open with')).toBe(shown);
        expect(!!screen.queryByLabelText('Arguments')).toBe(shown);
    });

    it('saves both, and drops them once the category no longer takes them', async () => {
        const onSave = jest.fn();
        render(<CommandForm onSave={onSave} onCancel={jest.fn()} />);
        await userEvent.type(screen.getByLabelText(/name/i), 'Private');
        await userEvent.type(screen.getByLabelText('URL *'), 'https://a.dev');
        await userEvent.type(screen.getByLabelText('Open with'), ' chrome.exe ');
        await userEvent.type(screen.getByLabelText('Arguments'), '--incognito');
        await userEvent.click(screen.getByRole('button', { name: 'Add' }));
        expect(onSave).toHaveBeenLastCalledWith(
            expect.objectContaining({ openWith: 'chrome.exe', openArgs: '--incognito' }),
        );

        await userEvent.selectOptions(screen.getByLabelText(/category/i), 'command');
        await userEvent.click(screen.getByRole('button', { name: 'Add' }));
        expect(onSave).toHaveBeenLastCalledWith(
            expect.objectContaining({ openWith: undefined, openArgs: undefined }),
        );
    });
});

describe('CommandForm run as administrator', () => {
    it.each([
        ['url', false],
        ['file', false],
        ['command', true],
    ])('shows the checkbox for %s: %s', async (category, shown) => {
        render(<CommandForm onSave={jest.fn()} onCancel={jest.fn()} />);
        await userEvent.selectOptions(screen.getByLabelText(/category/i), category);
        expect(!!screen.queryByLabelText('Run as administrator')).toBe(shown);
    });

    it('saves it for a command, and drops it after switching to url', async () => {
        const onSave = jest.fn();
        render(<CommandForm onSave={onSave} onCancel={jest.fn()} />);
        await userEvent.type(screen.getByLabelText(/name/i), 'Sessions');
        await userEvent.selectOptions(screen.getByLabelText(/category/i), 'command');
        await userEvent.type(screen.getByLabelText('Command *'), 'net session');
        await userEvent.click(screen.getByLabelText('Run as administrator'));
        await userEvent.click(screen.getByRole('button', { name: 'Add' }));
        expect(onSave).toHaveBeenLastCalledWith(
            expect.objectContaining({ category: 'command', runAsAdmin: true }),
        );

        await userEvent.selectOptions(screen.getByLabelText(/category/i), 'url');
        await userEvent.clear(screen.getByLabelText('URL *'));
        await userEvent.type(screen.getByLabelText('URL *'), 'https://a.dev');
        await userEvent.click(screen.getByRole('button', { name: 'Add' }));
        expect(onSave).toHaveBeenLastCalledWith(
            expect.objectContaining({ category: 'url', runAsAdmin: undefined }),
        );
    });
});
