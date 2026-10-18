import React, { useEffect } from 'react';
import { usePreventHide } from '../hooks/usePreventHide';

interface ModalProps {
    open: boolean;
    /** The element labelling the dialog, for screen readers. */
    titleId: string;
    /** Lets the caller close the dialog from its buttons: closing is what calls `onClose`. */
    dialogRef: React.RefObject<HTMLDialogElement | null>;
    /** Called whenever the dialog closes, whoever closed it: a button, Esc or the backdrop. */
    onClose: () => void;
    children: React.ReactNode;
}

/** The native <dialog> shell the app's modal dialogs share. */
export const Modal: React.FC<ModalProps> = ({ open, titleId, dialogRef, onClose, children }) => {
    usePreventHide(open);

    useEffect(() => {
        if (open) {
            dialogRef.current?.showModal();
        } else {
            dialogRef.current?.close();
        }
    }, [open, dialogRef]);

    useEffect(() => {
        const dialog = dialogRef.current;
        if (!dialog) return;
        dialog.addEventListener('close', onClose);
        return () => dialog.removeEventListener('close', onClose);
    }, [onClose, dialogRef]);

    return (
        <dialog ref={dialogRef} className="modal" aria-labelledby={titleId}>
            <div className="modal-box rounded-lg p-4 w-96 max-w-none">{children}</div>
            <form method="dialog" className="modal-backdrop">
                <button tabIndex={-1} aria-label="Close">
                    close
                </button>
            </form>
        </dialog>
    );
};
