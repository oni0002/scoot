import '@testing-library/jest-dom';

jest.mock('@tauri-apps/api/core', () => ({ invoke: jest.fn() }));
jest.mock('@tauri-apps/api/app', () => ({ getVersion: jest.fn().mockResolvedValue('0.0.0') }));
jest.mock('@tauri-apps/plugin-dialog', () => ({ message: jest.fn() }));
jest.mock('@tauri-apps/plugin-global-shortcut', () => ({ register: jest.fn(), unregister: jest.fn() }));
jest.mock('@tauri-apps/api/event', () => ({ listen: jest.fn().mockResolvedValue(() => {}), emit: jest.fn() }));
