// The Rust AppError Display adds a category prefix ("Validation error: ", ...). Users need the
// message itself, not the internal category.
const BACKEND_PREFIX =
    /^(?:Validation error|Command execution error|System error|Not found|IO error|JSON error): /;

export function getErrorMessage(err: unknown): string {
    const message =
        err instanceof Error ? err.message : typeof err === 'string' ? err : 'Unknown error';
    return message.replace(BACKEND_PREFIX, '');
}
