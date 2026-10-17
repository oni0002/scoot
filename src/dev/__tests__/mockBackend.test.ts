import { invoke } from '@tauri-apps/api/core';
import { TauriAPI } from '../../tauri';
import { createMockHandlers } from '../mockBackend';

// Guards against the mock drifting from the real IPC surface: every TauriAPI method that goes
// through invoke() must reach a mock handler. getVersion and showMessage use plugins, which
// setupTests mocks separately.
const PLUGIN_BACKED = ['getVersion', 'showMessage'];
const methods = Object.keys(TauriAPI).filter((name) => !PLUGIN_BACKED.includes(name));

type Method = (...args: unknown[]) => Promise<unknown>;

describe('mock backend', () => {
    let handlers: ReturnType<typeof createMockHandlers>;

    beforeEach(() => {
        handlers = createMockHandlers();
        jest.mocked(invoke).mockImplementation(async (cmd, args) => {
            const handler = handlers[cmd];
            if (!handler) throw new Error(`unhandled command: ${cmd}`);
            return handler((args ?? {}) as Record<string, unknown>);
        });
    });

    it('covers a meaningful number of TauriAPI methods', () => {
        expect(methods.length).toBeGreaterThan(10);
    });

    it.each(methods)('TauriAPI.%s reaches a mock handler', async (name) => {
        const error = await (TauriAPI as unknown as Record<string, Method>)[name]('u1', []).then(
            () => null,
            (e: unknown) => e,
        );
        expect(String(error)).not.toMatch(/unhandled command/);
    });

    it('rejects a duplicate alias like the real backend', () => {
        const base = { name: 'x', category: 'url', command: 'https://x.dev', description: '' };
        expect(() => handlers.add_command({ command: { ...base, id: '', alias: 'd' } })).toThrow(
            /already used/,
        );
    });

    it('fails execution of a missing file', async () => {
        await expect(handlers.execute_command({ id: 'u5' })).rejects.toThrow(/File not found/);
    });
});
