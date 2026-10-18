import React, { useId, useRef, useEffect } from 'react';
import { usePreventHide } from '../hooks/usePreventHide';

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

    usePreventHide(isOpen);

    useEffect(() => {
        if (isOpen) {
            dialogRef.current?.showModal();
        } else {
            dialogRef.current?.close();
        }
    }, [isOpen]);

    useEffect(() => {
        const dialog = dialogRef.current;
        if (!dialog) return;

        const handleClose = () => {
            onCancel();
        };

        dialog.addEventListener('close', handleClose);
        return () => dialog.removeEventListener('close', handleClose);
    }, [onCancel]);

    const handleCancel = () => {
        dialogRef.current?.close();
    };

    const handleConfirm = () => {
        onConfirm();
        dialogRef.current?.close();
    };

    return (
        <dialog ref={dialogRef} className="modal" aria-labelledby={titleId}>
            <div className="modal-box rounded-lg p-4 w-96 max-w-none">
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
            </div>
            <form method="dialog" className="modal-backdrop">
                <button tabIndex={-1} aria-label="Close">
                    close
                </button>
            </form>
        </dialog>
    );
};
