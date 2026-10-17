import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import '@testing-library/jest-dom';
import { CommandForm } from '../CommandForm';

jest.mock('@tauri-apps/plugin-dialog', () => ({ open: jest.fn() }));
jest.mock('../../tauri');

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
        expect(screen.getByRole('alert')).toHaveTextContent('Discard your changes?');
        expect(onCancel).not.toHaveBeenCalled();

        await userEvent.click(screen.getByRole('button', { name: 'Keep editing' }));
        expect(screen.queryByRole('alert')).not.toBeInTheDocument();
    });

    it('a second Esc or Discard confirms', async () => {
        const onCancel = jest.fn();
        render(<CommandForm onSave={jest.fn()} onCancel={onCancel} />);
        await userEvent.type(screen.getByLabelText(/name/i), 'x');
        await userEvent.keyboard('{Escape}{Escape}');
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
