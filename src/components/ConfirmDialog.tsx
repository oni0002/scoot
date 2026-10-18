import React, { useId, useRef } from 'react';
import { Modal } from './Modal';

interface ConfirmDialogProps {
    isOpen: boolean;
    title: string;
    children: React.ReactNode;
    confirmLabel: string;
    cancelLabel?: string;
    /** Styles the confirm button as destructive. */
    destructive?: boolean;
    onConfirm: () => void;
    /** Called whenever the dialog closes, including after a confirm. Esc and the backdrop cancel. */
    onCancel: () => void;
}

/** Modal yes/no question. Cancel has the initial focus, so Enter never confirms by accident. */
export const ConfirmDialog: React.FC<ConfirmDialogProps> = ({
    isOpen,
    title,
    children,
    confirmLabel,
    cancelLabel = 'Cancel',
    destructive = false,
    onConfirm,
    onCancel,
}) => {
    const dialogRef = useRef<HTMLDialogElement>(null);
    const titleId = useId();

    const handleCancel = () => {
        dialogRef.current?.close();
    };

    const handleConfirm = () => {
        onConfirm();
        dialogRef.current?.close();
    };

    return (
        <Modal open={isOpen} titleId={titleId} dialogRef={dialogRef} onClose={onCancel}>
            <h3 id={titleId} className="mb-2">
                {title}
            </h3>
            <p className="py-2 text-sm">{children}</p>

            <div className="modal-action">
                <button
                    type="button"
                    onClick={handleCancel}
                    className="btn btn-sm btn-ghost"
                    autoFocus
                >
                    {cancelLabel}
                </button>
                <button
                    type="button"
                    onClick={handleConfirm}
                    className={`btn btn-sm ${destructive ? 'btn-error' : 'btn-primary'}`}
                >
                    {confirmLabel}
                </button>
            </div>
        </Modal>
    );
};
