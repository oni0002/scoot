import React from 'react';
import { Command } from '../types';
import { ConfirmDialog } from './ConfirmDialog';

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
}) => (
    <ConfirmDialog
        isOpen={isOpen}
        title="Delete Command"
        confirmLabel="Delete"
        destructive
        onConfirm={onConfirm}
        onCancel={onCancel}
    >
        Are you sure you want to delete <span className="font-semibold">{command?.name}</span>?
        <br />
        This action cannot be undone.
    </ConfirmDialog>
);
