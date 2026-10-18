import { act, renderHook } from '@testing-library/react';
import { useNotifications } from '../useNotifications';

describe('useNotifications reportError', () => {
    beforeEach(() => jest.spyOn(console, 'error').mockImplementation(() => {}));
    afterEach(() => jest.restoreAllMocks());

    it('shows what failed and why, and logs the original error', () => {
        const { result } = renderHook(() => useNotifications());
        const cause = new Error('Access is denied');
        act(() => {
            result.current.reportError('open settings.json', cause, 1234);
        });

        const [toast] = result.current.notifications;
        expect(toast).toMatchObject({
            type: 'error',
            message: 'Could not open settings.json: Access is denied',
            duration: 1234,
        });
        expect(console.error).toHaveBeenCalledWith('Failed to open settings.json:', cause);
    });

    it('drops the backend error category from a string error', () => {
        const { result } = renderHook(() => useNotifications());
        act(() => {
            result.current.reportError('ignore "Zed"', 'Not found: Command not found');
        });
        expect(result.current.notifications[0].message).toBe(
            'Could not ignore "Zed": Command not found',
        );
    });
});
