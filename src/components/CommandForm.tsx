import React, { useState, useEffect, useCallback } from 'react';
import { Command } from '../types';
import { LuFile, LuFolder, LuArrowLeft } from 'react-icons/lu';
import { TauriAPI } from '../tauri';
import { hasPlaceholders } from '../command';
import { ConfirmDialog } from './ConfirmDialog';
import { useEscapeToClose } from '../hooks/useEscapeToClose';

interface CommandFormProps {
    command?: Command; // If provided, we're editing; otherwise, we're adding
    template?: Command; // Prefills a new command from an existing one (duplicate)
    onSave: (command: Command) => void;
    onCancel: () => void;
}

// A duplicate starts from the original's values but is a new command: no id, and no alias since
// an alias must stay unique. The name gets " copy" so the two rows can be told apart.
const toFormData = (command?: Command, template?: Command) => {
    const source = command ?? template;
    return {
        id: command?.id ?? '',
        name: command ? command.name : template ? `${template.name} copy` : '',
        category: source?.category ?? 'url',
        command: source?.command ?? '',
        description: source?.description ?? '',
        alias: command?.alias ?? '',
        workingDir: source?.workingDir ?? '',
        showWindow: source?.showWindow ?? false,
    };
};

export const CommandForm: React.FC<CommandFormProps> = ({
    command,
    template,
    onSave,
    onCancel,
}) => {
    const [formData, setFormData] = useState(() => toFormData(command, template));
    const [confirmingDiscard, setConfirmingDiscard] = useState(false);
    const [errors, setErrors] = useState<Record<string, string>>({});

    useEffect(() => {
        // eslint-disable-next-line react-hooks/set-state-in-effect -- initializing form state from props is intentional
        setFormData(toFormData(command, template));
        setErrors({});
        setConfirmingDiscard(false);
    }, [command, template]);

    const dirty = JSON.stringify(formData) !== JSON.stringify(toFormData(command, template));

    // Leaving with unsaved edits asks first; the dialog's own Esc means "keep editing".
    const requestCancel = useCallback(() => {
        if (dirty) {
            setConfirmingDiscard(true);
        } else {
            onCancel();
        }
    }, [dirty, onCancel]);

    const validateForm = (): boolean => {
        const newErrors: Record<string, string> = {};

        if (!formData.name.trim()) {
            newErrors.name = 'Name is required';
        }

        if (!formData.command.trim()) {
            newErrors.command =
                formData.category === 'url'
                    ? 'URL is required'
                    : formData.category === 'file'
                      ? 'Path is required'
                      : 'Command is required';
        }

        if (hasPlaceholders(formData.command) && !formData.alias.trim()) {
            newErrors.alias = 'Required for args';
        }

        setErrors(newErrors);
        return Object.keys(newErrors).length === 0;
    };

    const handleSubmit = (e: React.FormEvent) => {
        e.preventDefault();

        if (!validateForm()) {
            return;
        }

        const isCmdCategory = formData.category.trim() === 'command';

        const commandToSave: Command = {
            id: formData.id,
            name: formData.name.trim(),
            category: formData.category.trim(),
            command: formData.command.trim(),
            description: formData.description.trim(),
            alias: formData.alias.trim() || undefined,
            workingDir: isCmdCategory ? formData.workingDir.trim() || undefined : undefined,
            showWindow: isCmdCategory ? formData.showWindow : undefined,
        };

        onSave(commandToSave);
    };

    const handleInputChange = (field: keyof typeof formData, value: string | boolean) => {
        setFormData((prev) => ({ ...prev, [field]: value }));

        // Clear error when user starts typing
        if (errors[field]) {
            setErrors((prev) => ({ ...prev, [field]: '' }));
        }
    };

    // The window cannot hide while the form is open (see usePreventHide in App), so the native
    // dialog needs no modal guard of its own.
    const browse = async (directory: boolean) => {
        try {
            const selected = await TauriAPI.pickPath(directory);
            if (selected) handleInputChange('command', selected);
        } catch (err) {
            console.error(`Failed to open ${directory ? 'folder' : 'file'} dialog:`, err);
        }
    };

    useEscapeToClose(requestCancel, !confirmingDiscard);

    return (
        <div className="h-screen w-screen flex flex-col bg-base-100 text-base-content">
            {/* Header */}
            <div className="flex-none p-4 flex items-center gap-2">
                <button
                    onClick={requestCancel}
                    className="btn btn-ghost btn-sm btn-square"
                    title="Back"
                    aria-label="Back"
                >
                    <LuArrowLeft />
                </button>
                <h3>
                    {command ? 'Edit command' : template ? 'Duplicate command' : 'Add new command'}
                </h3>
            </div>

            {/* Body */}
            <form onSubmit={handleSubmit} className="flex-1 flex flex-col min-h-0">
                <div className="flex-1 overflow-y-auto px-4 space-y-2">
                    <div className="flex gap-2">
                        {/* Name */}
                        <div className="flex flex-col flex-1">
                            <label
                                htmlFor="cf-name"
                                className="flex select-none items-center px-1 py-2 justify-between"
                            >
                                <span className="text-sm">Name *</span>
                            </label>
                            <input
                                id="cf-name"
                                autoComplete="off"
                                type="text"
                                value={formData.name}
                                onChange={(e) => handleInputChange('name', e.target.value)}
                                placeholder="Enter command name"
                                className={`input input-sm w-full ${errors.name ? 'input-error' : ''}`}
                                autoFocus
                            />
                            {errors.name && (
                                <p
                                    role="alert"
                                    className="flex items-center px-1 py-2 text-xs text-error"
                                >
                                    {errors.name}
                                </p>
                            )}
                        </div>

                        {/* Category */}
                        <div className="flex flex-col w-32">
                            <label
                                htmlFor="cf-category"
                                className="flex select-none items-center px-1 py-2 justify-between"
                            >
                                <span className="text-sm">Category *</span>
                            </label>
                            <select
                                id="cf-category"
                                value={formData.category}
                                onChange={(e) => handleInputChange('category', e.target.value)}
                                className="select select-sm w-full"
                            >
                                <option value="url">URL</option>
                                <option value="command">Cmd</option>
                                <option value="file">File</option>
                            </select>
                        </div>
                    </div>

                    <div className="flex gap-2 items-start">
                        <div className="flex flex-col flex-1">
                            <label
                                htmlFor="cf-command"
                                className="flex select-none items-center px-1 py-2 justify-between"
                            >
                                <span className="text-sm">
                                    {formData.category === 'url'
                                        ? 'URL *'
                                        : formData.category === 'file'
                                          ? 'Path *'
                                          : 'Command *'}
                                </span>
                            </label>
                            <div className="flex gap-2 w-full">
                                <input
                                    id="cf-command"
                                    autoComplete="off"
                                    type="text"
                                    value={formData.command}
                                    onChange={(e) => handleInputChange('command', e.target.value)}
                                    placeholder={
                                        formData.category === 'url'
                                            ? 'https://example.com'
                                            : formData.category === 'file'
                                              ? 'C:\\path\\to\\file.txt'
                                              : 'Command line'
                                    }
                                    className={`input input-sm flex-1 ${errors.command ? 'input-error' : ''}`}
                                />
                                {formData.category === 'file' && (
                                    <>
                                        <button
                                            type="button"
                                            onClick={() => browse(false)}
                                            className="btn btn-square btn-sm btn-ghost"
                                            title="Select File"
                                            aria-label="Select file"
                                        >
                                            <LuFile />
                                        </button>
                                        <button
                                            type="button"
                                            onClick={() => browse(true)}
                                            className="btn btn-square btn-sm btn-ghost"
                                            title="Select Directory"
                                            aria-label="Select directory"
                                        >
                                            <LuFolder />
                                        </button>
                                    </>
                                )}
                            </div>
                            {errors.command && (
                                <p
                                    role="alert"
                                    className="flex items-center px-1 py-2 text-xs text-error"
                                >
                                    {errors.command}
                                </p>
                            )}
                            <p className="flex px-1 py-2 text-xs text-muted">
                                Use {'{$1}'}, {'{$2}'} or {'{$*}'} for arguments
                            </p>
                        </div>

                        <div className="flex flex-col w-32">
                            <label
                                htmlFor="cf-alias"
                                className="flex select-none items-center px-1 py-2 justify-between"
                            >
                                <span className="text-sm">
                                    Alias {hasPlaceholders(formData.command) ? '*' : ''}
                                </span>
                            </label>
                            <input
                                id="cf-alias"
                                autoComplete="off"
                                type="text"
                                value={formData.alias}
                                onChange={(e) => handleInputChange('alias', e.target.value)}
                                placeholder="e.g., g"
                                maxLength={10}
                                className={`input input-sm w-full ${errors.alias ? 'input-error' : ''}`}
                            />
                            <p className="px-1 py-1 text-xs text-muted">
                                {hasPlaceholders(formData.command)
                                    ? 'Type it, then the arguments'
                                    : 'Optional short name'}
                            </p>
                            {errors.alias && (
                                <p
                                    role="alert"
                                    className="flex items-center px-1 py-2 text-xs text-error"
                                >
                                    {errors.alias}
                                </p>
                            )}
                        </div>
                    </div>

                    {formData.category === 'command' && (
                        <>
                            <div className="flex flex-col w-full">
                                <label
                                    htmlFor="cf-working-dir"
                                    className="flex select-none items-center px-1 py-2 justify-between"
                                >
                                    <span className="text-sm">Working Dir</span>
                                </label>
                                <input
                                    id="cf-working-dir"
                                    autoComplete="off"
                                    type="text"
                                    value={formData.workingDir}
                                    onChange={(e) =>
                                        handleInputChange('workingDir', e.target.value)
                                    }
                                    placeholder="C:\path\to\working_dir"
                                    className="input input-sm w-full"
                                />
                            </div>

                            <div className="flex flex-col w-full">
                                <label className="flex select-none items-center px-1 py-2 cursor-pointer justify-start gap-4">
                                    <span className="text-sm">Show Window</span>
                                    <input
                                        type="checkbox"
                                        checked={formData.showWindow}
                                        onChange={(e) =>
                                            handleInputChange('showWindow', e.target.checked)
                                        }
                                        className="checkbox checkbox-sm"
                                    />
                                </label>
                            </div>
                        </>
                    )}

                    <div className="flex flex-col w-full">
                        <label
                            htmlFor="cf-description"
                            className="flex select-none items-center px-1 py-2 justify-between"
                        >
                            <span className="text-sm">Description</span>
                        </label>
                        <input
                            id="cf-description"
                            autoComplete="off"
                            type="text"
                            value={formData.description}
                            onChange={(e) => handleInputChange('description', e.target.value)}
                            placeholder="Brief description"
                            className="input input-sm w-full"
                        />
                    </div>
                </div>

                <div className="flex-none flex justify-end gap-2 p-4">
                    <button type="button" onClick={requestCancel} className="btn btn-ghost btn-sm">
                        Cancel
                    </button>
                    <button type="submit" className="btn btn-primary btn-sm">
                        {command ? 'Update' : 'Add'}
                    </button>
                </div>
            </form>

            <ConfirmDialog
                isOpen={confirmingDiscard}
                title="Discard changes?"
                confirmLabel="Discard"
                cancelLabel="Keep editing"
                destructive
                onConfirm={onCancel}
                onCancel={() => setConfirmingDiscard(false)}
            >
                Your edits to this command have not been saved.
            </ConfirmDialog>
        </div>
    );
};
