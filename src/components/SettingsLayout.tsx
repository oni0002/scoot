import React, { useId, useState } from 'react';
import { LuCheck, LuX } from 'react-icons/lu';

interface SectionProps {
    title: string;
    /** The element id, for a screen that opens scrolled to this section. */
    id?: string;
    /** Adds an on/off switch to the heading; the section's controls are disabled while it is off. */
    enabled?: { value: boolean; onChange: (value: boolean) => void };
    children: React.ReactNode;
}

/** A titled group of settings on one tinted surface, rows separated by hairlines. */
export const Section: React.FC<SectionProps> = ({ title, id, enabled, children }) => {
    const headingId = useId();
    return (
        <section id={id} aria-labelledby={headingId} className="mt-6 first:mt-2">
            <div className="mb-2 flex min-h-8 items-center justify-between px-3">
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

/** リストの値を載せる面・行の区切り・行の余白と文字・Add ボタン・削除ボタン。単列のリストと Files and Folders の表が同じ見た目になるよう、両方がここを使う */
export const LIST_SURFACE = 'rounded-md bg-base-100';
export const ROW_DIVIDER = 'border-base-content/10';
const LIST_ROW = `flex items-center gap-2 border-t px-2 py-1 text-[0.6875rem] first:border-t-0 ${ROW_DIVIDER}`;
export const ADD_BUTTON = 'btn btn-primary btn-xs mt-2';

/** 値を消すボタン。name は押す対象の名前(アクセシブルな名前に使う) */
export const RemoveButton: React.FC<{ name: string; onClick: () => void }> = ({
    name,
    onClick,
}) => (
    <button
        type="button"
        className="btn btn-ghost btn-xs btn-square flex-none"
        onClick={onClick}
        aria-label={`Remove ${name}`}
        title="Remove"
    >
        <LuX size={14} />
    </button>
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
            <ul className={`mt-2 ${LIST_SURFACE}`} aria-labelledby={labelId}>
                {items.map((item, i) => (
                    // Paths wrap instead of truncating: the end of a path is the part that matters
                    <li key={i} className={LIST_ROW}>
                        <span className="min-w-0 flex-1 break-all font-mono leading-5">{item}</span>
                        <RemoveButton name={item} onClick={() => onRemove(i)} />
                    </li>
                ))}
                {items.length === 0 && emptyText && (
                    // 項目の行は削除ボタンで 32px になる。空の行も同じ高さにする
                    <li className={`${LIST_ROW} min-h-8 text-subtle`}>{emptyText}</li>
                )}
                {extra}
            </ul>
            {addLabel && (
                <button type="button" className={ADD_BUTTON} onClick={onAdd} disabled={addDisabled}>
                    {addLabel}
                </button>
            )}
        </div>
    );
};

interface TextListRowProps extends Omit<ListRowProps, 'onAdd' | 'addDisabled' | 'extra'> {
    addLabel: string;
    /** 新しい値を入力する欄のアクセシブルな名前 */
    inputLabel: string;
    placeholder: string;
    /** 前後の空白を除いた、空でない入力を受け取る */
    onAdd: (text: string) => void;
}

/** Add でその場に入力欄を開くリスト。Enter・OK で足し、Esc・Cancel で閉じる */
export const TextListRow: React.FC<TextListRowProps> = ({
    inputLabel,
    placeholder,
    onAdd,
    ...row
}) => {
    // 欄を閉じている間は null
    const [draft, setDraft] = useState<string | null>(null);
    const close = () => setDraft(null);
    const confirm = () => {
        const text = draft?.trim();
        if (text) onAdd(text);
        close();
    };

    return (
        <ListRow
            {...row}
            onAdd={() => setDraft('')}
            addDisabled={draft !== null}
            extra={
                draft !== null && (
                    <li className={LIST_ROW}>
                        <input
                            autoFocus
                            autoComplete="off"
                            className="input input-xs flex-1 min-w-0 bg-transparent"
                            placeholder={placeholder}
                            aria-label={inputLabel}
                            value={draft}
                            onChange={(e) => setDraft(e.target.value)}
                            onKeyDown={(e) => {
                                if (e.key === 'Enter') confirm();
                                if (e.key === 'Escape') {
                                    // Only this field is closed, not the screen around it
                                    e.preventDefault();
                                    close();
                                }
                            }}
                        />
                        <button
                            type="button"
                            className="btn btn-ghost btn-xs btn-square flex-none"
                            onClick={confirm}
                            aria-label="OK"
                            title="OK"
                        >
                            <LuCheck size={14} />
                        </button>
                        <button
                            type="button"
                            className="btn btn-ghost btn-xs btn-square flex-none"
                            onClick={close}
                            aria-label="Cancel"
                            title="Cancel"
                        >
                            <LuX size={14} />
                        </button>
                    </li>
                )
            }
        />
    );
};
