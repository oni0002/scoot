const PLACEHOLDER_RE = /\{\$(\d+|\*)\}/;

export function hasPlaceholders(cmd: string): boolean {
    return PLACEHOLDER_RE.test(cmd);
}

export function substituteArgs(cmd: string, args: string[]): string {
    if (args.length === 0) return cmd;

    if (cmd.includes('{$*}')) {
        return cmd.replace('{$*}', args.join(' '));
    }

    let result = cmd;
    args.forEach((arg, index) => {
        result = result.replace(`{$${index + 1}}`, arg);
    });
    return result;
}
