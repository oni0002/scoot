import { listen } from '@tauri-apps/api/event';

/**
 * Subscribe to a backend event and get back the function that ends the subscription. It is safe
 * to call at once, before `listen` has finished: the subscription is then ended as soon as it
 * exists (keeping the result of a bare `listen().then(...)` would leak it).
 */
export function listenTo<T = unknown>(event: string, handler: (payload: T) => void): () => void {
    let cancelled = false;
    let unlisten: (() => void) | undefined;
    listen<T>(event, (e) => handler(e.payload)).then(
        (fn) => {
            if (cancelled) fn();
            else unlisten = fn;
        },
        (err) => console.warn(`Failed to listen to "${event}":`, err),
    );
    return () => {
        cancelled = true;
        unlisten?.();
    };
}
