// Matches an abbreviation against the starts of words: "vscode" is "V" + "S" + "code" in
// "Visual Studio Code". Fuse cannot see this kind of match, so search combines the two.

export interface BoundaryMatch {
    /** Inclusive [start, end] ranges of the matched characters, as Fuse reports them. */
    indices: [number, number][];
    /** How many words the query was spread over; fewer means a tighter match. */
    words: number;
}

const isAlnum = (c: string) => /[\p{L}\p{N}]/u.test(c);

/** A word starts after a non-alphanumeric, or at a lower-to-upper step (camelCase). */
function wordStarts(name: string): number[] {
    const starts: number[] = [];
    for (let i = 0; i < name.length; i++) {
        if (!isAlnum(name[i])) continue;
        const prev = name[i - 1];
        const camel =
            prev !== undefined &&
            prev === prev.toLowerCase() &&
            prev !== prev.toUpperCase() &&
            name[i] !== name[i].toLowerCase();
        if (prev === undefined || !isAlnum(prev) || camel) starts.push(i);
    }
    return starts;
}

/**
 * Whether `query` reads as a run of characters from the start of one word, then of the next
 * words, in order. Spaces in the query are ignored. Returns where it matched, or null.
 */
export function boundaryMatch(name: string, query: string): BoundaryMatch | null {
    const q = query.replace(/\s+/g, '').toLowerCase();
    if (!q) return null;
    const text = name.toLowerCase();
    const starts = wordStarts(name);

    // The matching of q from position qi, using word starts from index si on (null: none)
    const memo = new Map<number, [number, number][] | null>();
    const find = (qi: number, si: number): [number, number][] | null => {
        if (qi === q.length) return [];
        const key = qi * (starts.length + 1) + si;
        if (memo.has(key)) return memo.get(key) ?? null;

        let found: [number, number][] | null = null;
        for (let i = si; i < starts.length && !found; i++) {
            const start = starts[i];
            let run = 0;
            while (qi + run < q.length && text[start + run] === q[qi + run]) run++;
            // Prefer the longest run: it keeps the words used few
            for (; run >= 1 && !found; run--) {
                // The next word must begin after this run
                let next = i + 1;
                while (next < starts.length && starts[next] < start + run) next++;
                const rest = find(qi + run, next);
                if (rest) found = [[start, start + run - 1], ...rest];
            }
        }
        memo.set(key, found);
        return found;
    };

    const indices = find(0, 0);
    return indices ? { indices, words: indices.length } : null;
}
