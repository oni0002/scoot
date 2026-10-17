import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import '@testing-library/jest-dom';
import { NotificationToast } from '../NotificationToast';

describe('NotificationToast', () => {
    it('announces errors as alerts and the rest as status', () => {
        render(
            <NotificationToast
                onRemove={jest.fn()}
                notifications={[
                    { id: '1', type: 'error', message: 'Could not run "X"' },
                    { id: '2', type: 'success', message: 'Copied' },
                ]}
            />,
        );
        expect(screen.getByRole('alert')).toHaveTextContent('Could not run "X"');
        expect(screen.getByRole('status')).toHaveTextContent('Copied');
    });

    it('runs the toast action when its button is pressed', async () => {
        const onClick = jest.fn();
        render(
            <NotificationToast
                onRemove={jest.fn()}
                notifications={[
                    {
                        id: '1',
                        type: 'success',
                        message: 'Ignored "X"',
                        action: { label: 'Undo', onClick },
                    },
                ]}
            />,
        );
        await userEvent.click(screen.getByRole('button', { name: 'Undo' }));
        expect(onClick).toHaveBeenCalledTimes(1);
    });
});
