// {$1}, {$2}, ... or {$*}. Keep in sync with PLACEHOLDER_RE in src-tauri/src/commands/domain.rs.
const PLACEHOLDER_RE = /\{\$([1-9]\d*|\*)\}/;
const PLACEHOLDER_GLOBAL = /\{\$([1-9]\d*|\*)\}/g;

export function hasPlaceholders(cmd: string): boolean {
    return PLACEHOLDER_RE.test(cmd);
}

// Mirrors Command::substitute_args in src-tauri/src/commands/domain.rs (used for the preview only);
// both run the cases in src/__tests__/fixtures/substitute-cases.json
function percentEncode(s: string): string {
    return encodeURIComponent(s).replace(
        /[!'()*]/g,
        (c) => `%${c.charCodeAt(0).toString(16).toUpperCase()}`,
    );
}

/** One literal run or one placeholder of an expanded command template. */
export interface TemplatePart {
    text: string;
    /** The placeholder this part came from, e.g. "{$1}"; undefined for literal text. */
    slot?: string;
    /** True when the placeholder received a value (so `text` is the value, not the slot). */
    filled: boolean;
}

/** Splits a command into literal and placeholder parts, filling placeholders from args. */
export function expandTemplate(cmd: string, args: string[], urlEncode = false): TemplatePart[] {
    const values = urlEncode ? args.map(percentEncode) : args;
    const all = values.join(urlEncode ? '%20' : ' ');
    const parts: TemplatePart[] = [];
    let last = 0;

    for (const match of cmd.matchAll(PLACEHOLDER_GLOBAL)) {
        if (match.index > last) parts.push({ text: cmd.slice(last, match.index), filled: false });
        const slot = match[0];
        const value =
            match[1] === '*' ? (args.length > 0 ? all : undefined) : values[Number(match[1]) - 1];
        parts.push({ text: value ?? slot, slot, filled: value !== undefined });
        last = match.index + slot.length;
    }
    if (last < cmd.length) parts.push({ text: cmd.slice(last), filled: false });
    return parts;
}

export function substituteArgs(cmd: string, args: string[], urlEncode = false): string {
    return expandTemplate(cmd, args, urlEncode)
        .map((part) => part.text)
        .join('');
}

/** The arguments typed after an alias ("d foo bar" -> ["foo", "bar"]); none if the query is not for it. */
export function parseArgs(query: string, alias: string): string[] {
    const [first, ...rest] = query.trim().split(/\s+/);
    return first === alias ? rest : [];
}

/** The distinct placeholders a command takes, in order of appearance. */
export function slotsOf(cmd: string): string[] {
    return [...new Set([...cmd.matchAll(PLACEHOLDER_GLOBAL)].map((m) => m[0]))];
}

/** Placeholders that still have no value. */
export function missingSlots(cmd: string, args: string[]): string[] {
    return slotsOf(cmd).filter((slot) => {
        const key = slot.slice(2, -1);
        return key === '*' ? args.length === 0 : Number(key) > args.length;
    });
}

/** Typed arguments the command has no placeholder for (they would be silently dropped). */
export function unusedArgs(cmd: string, args: string[]): string[] {
    if (cmd.includes('{$*}')) return [];
    const highest = Math.max(0, ...slotsOf(cmd).map((slot) => Number(slot.slice(2, -1))));
    return args.slice(highest);
}
