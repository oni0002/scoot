import React, { useId } from 'react';
import { LuX } from 'react-icons/lu';

interface SectionProps {
    title: string;
    /** Adds an on/off switch to the heading; the section's controls are disabled while it is off. */
    enabled?: { value: boolean; onChange: (value: boolean) => void };
    children: React.ReactNode;
}

/** A titled group of settings on one tinted surface, rows separated by hairlines. */
export const Section: React.FC<SectionProps> = ({ title, enabled, children }) => {
    const headingId = useId();
    return (
        <section aria-labelledby={headingId} className="mt-6 first:mt-2">
            <div className="mb-2 flex min-h-8 items-center justify-between">
                <h4 id={headingId} className="text-sm font-semibold">
                    {title}
                </h4>
                {enabled && (
                    <input
                        type="checkbox"
                        className="toggle"
                        aria-label={`Enable ${title}`}
                        checked={enabled.value}
                        onChange={(e) => enabled.onChange(e.target.checked)}
                    />
                )}
            </div>
            {/* A disabled fieldset disables every control inside it, natively */}
            <fieldset
                disabled={enabled ? !enabled.value : false}
                className="min-w-0 divide-y divide-base-content/10 rounded-lg bg-base-content/[0.06] disabled:opacity-60"
            >
                {children}
            </fieldset>
        </section>
    );
};

interface RowProps {
    label: string;
    /** The id of the control the label names. */
    htmlFor?: string;
    description?: string;
    children: React.ReactNode;
}

/** One setting: name and help on the left, the control on the right. */
export const Row: React.FC<RowProps> = ({ label, htmlFor, description, children }) => (
    <div className="flex min-h-14 items-center gap-4 px-3 py-2.5">
        <div className="min-w-0 flex-1">
            <label htmlFor={htmlFor} className="block select-none text-sm">
                {label}
            </label>
            {description && <p className="text-xs text-subtle">{description}</p>}
        </div>
        <div className="flex-none">{children}</div>
    </div>
);

interface ListRowProps {
    label: string;
    description?: string;
    items: string[];
    onRemove: (index: number) => void;
    /** Shown when the list is empty. */
    emptyText?: string;
    addLabel?: string;
    onAdd?: () => void;
    addDisabled?: boolean;
    /** Extra rows after the items, e.g. an inline editor. */
    extra?: React.ReactNode;
}

/** A setting that is a list: header, then one row per value with a remove button, then Add. */
export const ListRow: React.FC<ListRowProps> = ({
    label,
    description,
    items,
    onRemove,
    emptyText,
    addLabel,
    onAdd,
    addDisabled,
    extra,
}) => {
    const labelId = useId();
    return (
        <div className="px-3 py-2.5">
            <p id={labelId} className="select-none text-sm">
                {label}
            </p>
            {description && <p className="text-xs text-subtle">{description}</p>}
            <ul className="mt-2 space-y-1" aria-labelledby={labelId}>
                {items.map((item, i) => (
                    // Paths wrap instead of truncating: the end of a path is the part that matters
                    <li
                        key={i}
                        className="flex items-start gap-2 rounded-md bg-base-100 py-1 pl-2 pr-1"
                    >
                        <span className="min-w-0 flex-1 break-all py-0.5 font-mono text-xs leading-5">
                            {item}
                        </span>
                        <button
                            type="button"
                            className="btn btn-ghost btn-xs btn-square flex-none"
                            onClick={() => onRemove(i)}
                            aria-label={`Remove ${item}`}
                        >
                            <LuX size={14} />
                        </button>
                    </li>
                ))}
                {items.length === 0 && emptyText && (
                    <li className="px-1 py-1 text-xs text-subtle">{emptyText}</li>
                )}
                {extra}
            </ul>
            {addLabel && (
                <button
                    type="button"
                    className="btn btn-sm mt-2 border-base-content/50 bg-base-100"
                    onClick={onAdd}
                    disabled={addDisabled}
                >
                    {addLabel}
                </button>
            )}
        </div>
    );
};
