import React, { useState, useEffect } from 'react';
import { Command } from '../types';
import { open } from '@tauri-apps/plugin-dialog';
import { LuFile, LuFolder, LuArrowLeft } from 'react-icons/lu';
import { usePreventHide } from '../hooks/usePreventHide';
import { TauriAPI } from '../api/tauri';

interface CommandFormProps {
    command?: Command; // If provided, we're editing; otherwise, we're adding
    onSave: (command: Command) => void;
    onCancel: () => void;
}

export const CommandForm: React.FC<CommandFormProps> = ({
    command,
    onSave,
    onCancel,
}) => {
    const [formData, setFormData] = useState({
        id: '',
        name: '',
        category: '',
        command: '',
        description: '',
        prompt: '',
        workingDir: '',
        showWindow: false,
    });
    const [errors, setErrors] = useState<Record<string, string>>({});

    // フォームが表示されている間はウィンドウ非表示を防ぐ
    usePreventHide(true);

    useEffect(() => {
        if (command) {
            // Editing existing command
            setFormData({
                id: command.id,
                name: command.name,
                category: command.category,
                command: command.command,
                description: command.description,
                prompt: command.prompt || '',
                workingDir: command.workingDir || '',
                showWindow: command.showWindow || false,
            });
        } else {
            // Adding new command
            setFormData({
                id: '',
                name: '',
                category: 'url',
                command: '',
                description: '',
                prompt: '',
                workingDir: '',
                showWindow: false,
            });
        }
        setErrors({});
    }, [command]);

    const validateForm = (): boolean => {
        const newErrors: Record<string, string> = {};

        if (!formData.name.trim()) {
            newErrors.name = 'Name is required';
        }

        if (!formData.category.trim()) {
            newErrors.category = 'Category is required';
        }

        if (!formData.command.trim()) {
            newErrors.command = 'Command is required';
        }

        const hasPlaceholders = /\{\$(\d+|\*)\}/.test(formData.command);
        if (hasPlaceholders && !formData.prompt.trim()) {
            newErrors.prompt = 'Required for args';
        }

        setErrors(newErrors);
        return Object.keys(newErrors).length === 0;
    };

    const handleSubmit = (e: React.FormEvent) => {
        e.preventDefault();

        if (!validateForm()) {
            return;
        }

        const isCmdCategory = formData.category.trim() === 'cmd';

        const commandToSave: Command = {
            id: formData.id,
            name: formData.name.trim(),
            category: formData.category.trim(),
            command: formData.command.trim(),
            description: formData.description.trim(),
            prompt: formData.prompt.trim() || undefined,
            workingDir: isCmdCategory ? (formData.workingDir.trim() || undefined) : undefined,
            showWindow: isCmdCategory ? formData.showWindow : undefined,
        };

        onSave(commandToSave);
    };

    const handleInputChange = (field: keyof typeof formData, value: string | boolean) => {
        setFormData(prev => {
            const newData = { ...prev, [field]: value };

            // Auto-clear prompt if command no longer has placeholders
            if (field === 'command' && typeof value === 'string') {
                const hasPlaceholders = /\{\$(\d+|\*)\}/.test(value);
                if (!hasPlaceholders) {
                    newData.prompt = '';
                }
            }
            return newData;
        });

        // Clear error when user starts typing
        if (errors[field]) {
            setErrors(prev => ({ ...prev, [field]: '' }));
        }
    };

    const handleBrowseFile = async () => {
        try {
            await TauriAPI.setPreventHide(true);
            const selected = await open({
                multiple: false,
                directory: false,
            });
            if (selected && typeof selected === 'string') {
                handleInputChange('command', selected);
            }
        } catch (err) {
            console.error('Failed to open file dialog:', err);
        } finally {
            await TauriAPI.setPreventHide(false);
            // フォームにフォーカスを戻す（コンテナ等へのフォーカス移動を検討）
        }
    };

    const handleBrowseFolder = async () => {
        try {
            await TauriAPI.setPreventHide(true);
            const selected = await open({
                multiple: false,
                directory: true,
            });
            if (selected && typeof selected === 'string') {
                handleInputChange('command', selected);
            }
        } catch (err) {
            console.error('Failed to open folder dialog:', err);
        } finally {
            await TauriAPI.setPreventHide(false);
        }
    };

    // Keyboard navigation for Escape key to cancel
    useEffect(() => {
        const handleKeyDown = (e: KeyboardEvent) => {
            if (e.key === 'Escape') {
                onCancel();
            }
        };
        window.addEventListener('keydown', handleKeyDown);
        return () => window.removeEventListener('keydown', handleKeyDown);
    }, [onCancel]);

    return (
        <div className="h-screen w-screen flex flex-col bg-base-100 text-base-content scrollbar-gutter-auto">
            <div className="flex-none p-4 flex items-center gap-2">
                <button onClick={onCancel} className="btn btn-ghost btn-sm btn-square" title="Back">
                    <LuArrowLeft />
                </button>
                <h3>{command ? 'Edit command' : 'Add new command'}</h3>
            </div>

            <form onSubmit={handleSubmit} className="flex-1 flex flex-col min-h-0">
                <div className="flex-1 overflow-y-auto px-4 scrollbar-gutter-auto space-y-2">
                    <div className="flex gap-2">
                        <div className="form-control flex-1">
                            <label className="label">
                                <span className="label-text">Name *</span>
                            </label>
                            <input
                                type="text"
                                value={formData.name}
                                onChange={(e) => handleInputChange('name', e.target.value)}
                                placeholder="Enter command name"
                                className={`input input-bordered input-sm w-full ${errors.name ? 'input-error' : ''}`}
                                autoFocus
                            />
                            {errors.name && (
                                <label className="label">
                                    <span className="label-text-alt text-error">{errors.name}</span>
                                </label>
                            )}
                        </div>

                        <div className="form-control w-32">
                            <label className="label">
                                <span className="label-text">Category *</span>
                            </label>
                            <select
                                value={formData.category}
                                onChange={(e) => handleInputChange('category', e.target.value)}
                                className={`select select-bordered select-sm w-full ${errors.category ? 'select-error' : ''}`}
                            >
                                <option value="url">URL</option>
                                <option value="command">Cmd</option>
                                <option value="file">File</option>
                            </select>
                            {errors.category && (
                                <label className="label">
                                    <span className="label-text-alt text-error">{errors.category}</span>
                                </label>
                            )}
                        </div>
                    </div>

                    <div className="flex gap-2 items-start">
                        <div className="form-control flex-1">
                            <label className="label">
                                <span className="label-text">
                                    {formData.category === 'url' ? 'URL *' :
                                        formData.category === 'file' ? 'Path *' :
                                            'Command *'}
                                </span>
                            </label>
                            <div className="flex gap-2 w-full">
                                <input
                                    type="text"
                                    value={formData.command}
                                    onChange={(e) => handleInputChange('command', e.target.value)}
                                    placeholder={
                                        formData.category === 'url' ? 'https://example.com' :
                                            formData.category === 'file' ? 'C:\\path\\to\\file.txt' :
                                                'Command line'
                                    }
                                    className={`input input-bordered input-sm flex-1 ${errors.command ? 'input-error' : ''}`}
                                />
                                {formData.category === 'file' && (
                                    <>
                                        <button
                                            type="button"
                                            onClick={handleBrowseFile}
                                            className="btn btn-square btn-sm btn-ghost"
                                            title="Select File"
                                        >
                                            <LuFile />
                                        </button>
                                        <button
                                            type="button"
                                            onClick={handleBrowseFolder}
                                            className="btn btn-square btn-sm btn-ghost"
                                            title="Select Directory"
                                        >
                                            <LuFolder />
                                        </button>
                                    </>
                                )}
                            </div>
                            {errors.command && (
                                <label className="label">
                                    <span className="label-text-alt text-error">{errors.command}</span>
                                </label>
                            )}
                            <label className="label">
                                <span className="label-text-alt">
                                    Use {'{$1}'}, {'{$2}'} or {'{$*}'} for arguments
                                </span>
                            </label>
                        </div>

                        <div className="form-control w-32">
                            <label className="label">
                                <span className="label-text">Prompt {/\{\$(\d+|\*)\}/.test(formData.command) ? '*' : ''}</span>
                            </label>
                            <input
                                type="text"
                                value={formData.prompt}
                                onChange={(e) => handleInputChange('prompt', e.target.value)}
                                placeholder="e.g., g"
                                maxLength={10}
                                className={`input input-bordered input-sm w-full ${errors.prompt ? 'input-error' : ''}`}
                                disabled={!/\{\$(\d+|\*)\}/.test(formData.command)}
                            />
                            {errors.prompt && (
                                <label className="label">
                                    <span className="label-text-alt text-error">{errors.prompt}</span>
                                </label>
                            )}
                        </div>
                    </div>

                    {formData.category === 'command' && (
                        <>
                            <div className="form-control w-full">
                                <label className="label">
                                    <span className="label-text">Working Dir</span>
                                </label>
                                <input
                                    type="text"
                                    value={formData.workingDir}
                                    onChange={(e) => handleInputChange('workingDir', e.target.value)}
                                    placeholder="C:\path\to\working_dir"
                                    className="input input-bordered input-sm w-full"
                                />
                            </div>

                            <div className="form-control w-full">
                                <label className="label cursor-pointer justify-start gap-4">
                                    <span className="label-text">Show Window</span>
                                    <input
                                        type="checkbox"
                                        checked={formData.showWindow}
                                        onChange={(e) => handleInputChange('showWindow', e.target.checked)}
                                        className="checkbox checkbox-sm"
                                    />
                                </label>
                            </div>
                        </>
                    )}

                    <div className="form-control w-full">
                        <label className="label">
                            <span className="label-text">Description</span>
                        </label>
                        <input
                            type="text"
                            value={formData.description}
                            onChange={(e) => handleInputChange('description', e.target.value)}
                            placeholder="Brief description"
                            className={`input input-bordered input-sm w-full ${errors.description ? 'input-error' : ''}`}
                        />
                    </div>
                </div>

                <div className="flex-none flex justify-end gap-2 p-4">
                    <button type="button" onClick={onCancel} className="btn btn-ghost btn-sm">
                        Cancel
                    </button>
                    <button type="submit" className="btn btn-primary btn-sm">
                        {command ? 'Update' : 'Add'}
                    </button>
                </div>
            </form >
        </div >
    );
};
