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
