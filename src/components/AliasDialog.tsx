import React, { useId, useRef, useState } from 'react';
import { Command } from '../types';
import { getErrorMessage } from '../error';
import { Modal } from './Modal';

interface AliasDialogProps {
    /** The external command to name; the dialog is open while this is set. */
    command?: Command;
    /** Saves the alias (null clears it). Rejecting keeps the dialog open and shows the reason. */
    onSubmit: (alias: string | null) => Promise<void>;
    onCancel: () => void;
}

const AliasDialogBody: React.FC<Required<AliasDialogProps>> = ({ command, onSubmit, onCancel }) => {
    const dialogRef = useRef<HTMLDialogElement>(null);
    const titleId = useId();
    const [value, setValue] = useState(command.alias ?? '');
    const [error, setError] = useState('');
    const [busy, setBusy] = useState(false);

    const submit = async (alias: string | null) => {
        setBusy(true);
        setError('');
        try {
            await onSubmit(alias);
        } catch (err) {
            setError(getErrorMessage(err));
            setBusy(false);
        }
    };

    const handleSubmit = (e: React.FormEvent) => {
        e.preventDefault();
        submit(value.trim() || null);
    };

    return (
        <Modal open titleId={titleId} dialogRef={dialogRef} onClose={onCancel}>
            <form onSubmit={handleSubmit}>
                <h3 id={titleId} className="mb-2">
                    Alias
                </h3>
                <label htmlFor="alias-dialog-input" className="block px-1 py-2 text-sm truncate">
                    A short name for <span className="font-semibold">{command.name}</span>
                </label>
                <input
                    id="alias-dialog-input"
                    autoComplete="off"
                    autoFocus
                    type="text"
                    value={value}
                    maxLength={10}
                    placeholder="e.g., vscode"
                    onChange={(e) => {
                        setValue(e.target.value);
                        setError('');
                    }}
                    className={`input input-sm w-full ${error ? 'input-error' : ''}`}
                />
                {error ? (
                    <p role="alert" className="px-1 py-2 text-xs text-error">
                        {error}
                    </p>
                ) : (
                    <p className="px-1 py-2 text-xs text-muted">Search for it by this name</p>
                )}

                <div className="modal-action">
                    {command.alias && (
                        <button
                            type="button"
                            disabled={busy}
                            onClick={() => submit(null)}
                            className="btn btn-sm btn-ghost text-error mr-auto"
                        >
                            Remove
                        </button>
                    )}
                    <button
                        type="button"
                        onClick={() => dialogRef.current?.close()}
                        className="btn btn-sm btn-ghost"
                    >
                        Cancel
                    </button>
                    <button type="submit" disabled={busy} className="btn btn-sm btn-primary">
                        Save
                    </button>
                </div>
            </form>
        </Modal>
    );
};

/** Sets the alias of an external command, which has no edit form of its own. */
export const AliasDialog: React.FC<AliasDialogProps> = ({ command, onSubmit, onCancel }) =>
    command ? (
        <AliasDialogBody
            key={command.id}
            command={command}
            onSubmit={onSubmit}
            onCancel={onCancel}
        />
    ) : null;
