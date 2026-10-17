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

/** The three things a first-time user needs; the hint bar covers the rest once something is selected. */
export const QuickGuide: React.FC<{ aliasExample?: string }> = ({ aliasExample }) => {
    const rows: Array<[label: string, body: React.ReactNode]> = [
        [
            'Search',
            <span key="s">
                Type a keyword or a URL
                {aliasExample && (
                    <>
                        , or an alias like{' '}
                        <span className="rounded-sm bg-base-content/10 px-1 font-mono">
                            {aliasExample} text
                        </span>
                    </>
                )}
            </span>,
        ],
        [
            'Move',
            <span key="m" className="flex items-center gap-1">
                <Keys keys={['Tab']} />
                <Or />
                <Keys keys={['Shift', 'Tab']} />
            </span>,
        ],
        ['Run', <Keys key="r" keys={['Enter']} />],
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
