const PLACEHOLDER_RE = /\{\$(\d+|\*)\}/;

export function hasPlaceholders(cmd: string): boolean {
    return PLACEHOLDER_RE.test(cmd);
}

// Mirrors Command::substitute_args in src-tauri/src/commands/domain.rs (used for the preview only)
function percentEncode(s: string): string {
    return encodeURIComponent(s).replace(
        /[!'()*]/g,
        (c) => `%${c.charCodeAt(0).toString(16).toUpperCase()}`,
    );
}

export function substituteArgs(cmd: string, args: string[], urlEncode = false): string {
    if (args.length === 0) return cmd;

    const values = urlEncode ? args.map(percentEncode) : args;
    let result = cmd.split('{$*}').join(values.join(urlEncode ? '%20' : ' '));
    values.forEach((arg, index) => {
        result = result.split(`{$${index + 1}}`).join(arg);
    });
    return result;
}
