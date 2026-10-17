import React from 'react';

const Keys: React.FC<{ keys: string[] }> = ({ keys }) => (
    <span className="flex items-center gap-1">
        {keys.map((k, i) => (
            <React.Fragment key={i}>
                {i > 0 && <span aria-hidden="true">+</span>}
                <kbd className="kbd kbd-xs">{k}</kbd>
            </React.Fragment>
        ))}
    </span>
);

const Or: React.FC = () => <span aria-hidden="true">/</span>;

/** What the launcher can do, shown while the search field is empty. */
export const QuickGuide: React.FC<{ hotkey: string }> = ({ hotkey }) => {
    const rows: Array<[label: string, body: React.ReactNode]> = [
        ['Search', <span key="s">Type part of a name or alias</span>],
        [
            'Move',
            <span key="m" className="flex items-center gap-1">
                <Keys keys={['Tab']} />
                <Or />
                <Keys keys={['Shift', 'Tab']} />
            </span>,
        ],
        ['Run', <Keys key="r" keys={['Enter']} />],
        [
            'Arguments',
            <span key="a">
                Alias, space, then text: <span className="font-mono">g react</span>
            </span>,
        ],
        ['Open', <span key="o">Type a URL or a file path</span>],
        ['Copy', <Keys key="c" keys={['Ctrl', 'C']} />],
        ['Close', <Keys key="x" keys={['Esc']} />],
        ['Show / hide', <Keys key="h" keys={hotkey.split('+')} />],
    ];

    return (
        <div className="flex flex-col items-center justify-center py-8 text-muted h-full select-none">
            <dl className="grid grid-cols-[auto_1fr] items-center gap-x-4 gap-y-1.5 text-sm">
                {rows.map(([label, body]) => (
                    <React.Fragment key={label}>
                        <dt className="text-right">{label}</dt>
                        <dd className="text-xs">{body}</dd>
                    </React.Fragment>
                ))}
            </dl>
        </div>
    );
};

/** Nothing matched: say how to get out of it. */
export const NoResults: React.FC<{ query: string }> = ({ query }) => (
    <div className="flex flex-col items-center justify-center gap-2 py-12 text-muted h-full select-none">
        <p>No commands found for &quot;{query}&quot;</p>
        <p className="flex items-center gap-1.5 text-xs">
            <kbd className="kbd kbd-xs">Esc</kbd> clears the search. To add one, use{' '}
            <span aria-hidden="true">⋮</span>
            <span className="sr-only">the More actions menu</span> › Add Command.
        </p>
    </div>
);
