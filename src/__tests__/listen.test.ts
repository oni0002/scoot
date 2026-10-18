import { listen } from '@tauri-apps/api/event';
import { listenTo } from '../listen';

describe('listenTo', () => {
    it('hands the payload to the handler', async () => {
        let deliver: (event: { payload: string }) => void = () => {};
        jest.mocked(listen).mockImplementationOnce(async (_event, cb) => {
            deliver = cb as typeof deliver;
            return () => {};
        });
        const handler = jest.fn();
        listenTo<string>('hotkey-captured', handler);
        await Promise.resolve();
        deliver({ payload: 'Alt+K' });
        expect(handler).toHaveBeenCalledWith('Alt+K');
    });

    it('ends the subscription when stopped after it was set up', async () => {
        const unlisten = jest.fn();
        jest.mocked(listen).mockResolvedValueOnce(unlisten);
        const stop = listenTo('window-shown', jest.fn());
        await Promise.resolve();
        stop();
        expect(unlisten).toHaveBeenCalledTimes(1);
    });

    it('ends the subscription that finishes after it was stopped', async () => {
        const unlisten = jest.fn();
        jest.mocked(listen).mockResolvedValueOnce(unlisten);
        const stop = listenTo('window-shown', jest.fn());
        stop(); // before listen() has resolved
        expect(unlisten).not.toHaveBeenCalled();
        await Promise.resolve();
        await Promise.resolve();
        expect(unlisten).toHaveBeenCalledTimes(1);
    });
});
