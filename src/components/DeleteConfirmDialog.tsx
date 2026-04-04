import React, { useRef, useEffect } from 'react';
import { Command } from '../types';
import { usePreventHide } from '../hooks/usePreventHide';

interface DeleteConfirmDialogProps {
  isOpen: boolean;
  command?: Command;
  onConfirm: () => void;
  onCancel: () => void;
}

export const DeleteConfirmDialog: React.FC<DeleteConfirmDialogProps> = ({
  isOpen,
  command,
  onConfirm,
  onCancel,
}) => {
  const dialogRef = useRef<HTMLDialogElement>(null);

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
    <dialog ref={dialogRef} className="modal scrollbar-gutter-auto">
      <div className="modal-box rounded-lg p-4 w-96 max-w-none scrollbar-gutter-auto">
        <h3 className="mb-2">Delete Command</h3>
        <p className="py-2 text-sm">
          Are you sure you want to delete <span className="font-semibold">{command?.name}</span>?
          <br />
          This action cannot be undone.
        </p>

        <div className="modal-action">
          <button
            type="button"
            onClick={handleCancel}
            className="btn btn-sm btn-ghost"
            autoFocus
          >
            Cancel
          </button>
          <button
            type="button"
            onClick={handleConfirm}
            className="btn btn-sm btn-error"
          >
            Delete
          </button>
        </div>
      </div>
      <form method="dialog" className="modal-backdrop">
        <button>close</button>
      </form>
    </dialog>
  );
};