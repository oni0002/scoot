import { render, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import '@testing-library/jest-dom';
import { invoke } from '@tauri-apps/api/core';
import App from './App';
import { createMockHandlers } from './dev/mockBackend';

// Mock the Tauri API
jest.mock('@tauri-apps/api/core', () => ({
    invoke: jest.fn(),
}));

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

describe('App', () => {
    test('renders loading state initially', () => {
        render(<App />);
        expect(screen.getByText('Loading Scoot...')).toBeInTheDocument();
    });
});

const calls = (name: string) =>
    jest.mocked(invoke).mock.calls.filter(([command]) => command === name);

const mockBackend = () => {
    const handlers = createMockHandlers();
    jest.mocked(invoke).mockReset();
    jest.mocked(invoke).mockImplementation(async (command, args) => {
        const handler = handlers[command];
        if (!handler) throw new Error(`unhandled command: ${command}`);
        return handler((args ?? {}) as Record<string, unknown>);
    });
};

describe('App deleting a command', () => {
    // Opens the confirmation for "GitHub": search for it, then Ctrl+D (the row's Delete action)
    const askToDelete = async () => {
        await userEvent.type(await screen.findByLabelText('Search commands'), 'GitHub');
        await userEvent.keyboard('{Control>}d{/Control}');
        return screen.findByRole('dialog');
    };

    beforeEach(mockBackend);

    test('deletes after confirming and says so', async () => {
        render(<App />);
        const dialog = await askToDelete();
        await userEvent.click(within(dialog).getByRole('button', { name: 'Delete' }));

        await waitFor(() => expect(calls('delete_command')).toHaveLength(1));
        expect(calls('delete_command')[0][1]).toEqual({ id: 'u3' });
        expect(await screen.findByText('"GitHub" deleted successfully.')).toBeInTheDocument();
        expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
    });

    test('deletes nothing when the question is dismissed', async () => {
        render(<App />);
        const dialog = await askToDelete();
        await userEvent.click(within(dialog).getByRole('button', { name: 'Cancel' }));

        expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
        expect(calls('delete_command')).toHaveLength(0);

        // The next question is about the next command, not a leftover one
        await userEvent.clear(screen.getByLabelText('Search commands'));
        await userEvent.type(screen.getByLabelText('Search commands'), 'download');
        await userEvent.keyboard('{Control>}d{/Control}');
        expect(await screen.findByRole('dialog')).toHaveTextContent('download');
    });
});

describe('App command form', () => {
    // Searches for "GitHub" and runs one of its row shortcuts (Ctrl+E edit, Ctrl+U duplicate)
    const openFromRow = async (key: string) => {
        await userEvent.type(await screen.findByLabelText('Search commands'), 'GitHub');
        await userEvent.keyboard(`{Control>}${key}{/Control}`);
    };

    const openAddForm = async () => {
        await userEvent.click(await screen.findByLabelText('More actions'));
        await userEvent.click(await screen.findByText('Add Command'));
    };

    beforeEach(mockBackend);

    test('edit opens the form filled with the command and saves it as an update', async () => {
        render(<App />);
        await openFromRow('e');

        expect(await screen.findByText('Edit command')).toBeInTheDocument();
        expect(screen.getByLabelText('Name *')).toHaveValue('GitHub');

        await userEvent.type(screen.getByLabelText('Name *'), '!');
        await userEvent.click(screen.getByRole('button', { name: 'Update' }));

        await waitFor(() => expect(calls('update_command')).toHaveLength(1));
        expect(calls('add_command')).toHaveLength(0);
        expect(await screen.findByText('"GitHub!" updated successfully.')).toBeInTheDocument();
        expect(screen.getByLabelText('Search commands')).toBeInTheDocument();
    });

    test('duplicate opens a copy with no id and saves it as a new command', async () => {
        render(<App />);
        await openFromRow('u');

        expect(await screen.findByText('Duplicate command')).toBeInTheDocument();
        expect(screen.getByLabelText('Name *')).toHaveValue('GitHub copy');

        await userEvent.click(screen.getByRole('button', { name: 'Add' }));

        await waitFor(() => expect(calls('add_command')).toHaveLength(1));
        expect(calls('update_command')).toHaveLength(0);
        expect(await screen.findByText('"GitHub copy" added successfully.')).toBeInTheDocument();
    });

    test('cancel returns to search, and the next form starts empty', async () => {
        render(<App />);
        await openFromRow('e');
        await screen.findByText('Edit command');
        await userEvent.click(screen.getByRole('button', { name: 'Cancel' }));

        expect(await screen.findByLabelText('Search commands')).toBeInTheDocument();

        await openAddForm();
        expect(await screen.findByText('Add new command')).toBeInTheDocument();
        expect(screen.getByLabelText('Name *')).toHaveValue('');
    });
});
