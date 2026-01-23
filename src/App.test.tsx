import { render, screen } from '@testing-library/react';
import App from './App';

// Mock the Tauri API
jest.mock('@tauri-apps/api/core', () => ({
  invoke: jest.fn(),
}));

describe('App', () => {
  test('renders loading state initially', () => {
    render(<App />);
    expect(screen.getByText('Loading Scoot...')).toBeInTheDocument();
  });
});