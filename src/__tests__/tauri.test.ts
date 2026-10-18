import { open } from '@tauri-apps/plugin-dialog';
import { TauriAPI } from '../tauri';

jest.mock('@tauri-apps/plugin-dialog', () => ({ open: jest.fn(), message: jest.fn() }));

describe('TauriAPI.pickPath', () => {
    it('asks for one file or one folder and returns what was chosen', async () => {
        jest.mocked(open).mockResolvedValue('C:\notes\todo.md');
        await expect(TauriAPI.pickPath(false)).resolves.toBe('C:\notes\todo.md');
        expect(open).toHaveBeenCalledWith({ multiple: false, directory: false });

        await TauriAPI.pickPath(true);
        expect(open).toHaveBeenLastCalledWith({ multiple: false, directory: true });
    });

    it('returns null when the user cancels', async () => {
        jest.mocked(open).mockResolvedValue(null);
        await expect(TauriAPI.pickPath(false)).resolves.toBeNull();
    });
});
