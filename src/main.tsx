import React from 'react';
import ReactDOM from 'react-dom/client';
import App from './App';

async function bootstrap() {
    // Dev only: in a plain browser there is no Tauri backend, so serve a mock one.
    // Stripped from production builds (import.meta.env.DEV is false there).
    if (import.meta.env.DEV && !('__TAURI_INTERNALS__' in window)) {
        const { installMockBackend } = await import('./dev/mockBackend');
        installMockBackend();
    }

    ReactDOM.createRoot(document.getElementById('root') as HTMLElement).render(
        <React.StrictMode>
            <App />
        </React.StrictMode>,
    );
}

bootstrap();
