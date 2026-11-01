import { useState, useCallback } from 'react';
import { SearchWindow } from './components/SearchWindow';
import { CommandForm } from './components/CommandForm';
import { SettingsScreen } from './components/SettingsScreen';
import { ConfirmDialog } from './components/ConfirmDialog';
import { AliasDialog } from './components/AliasDialog';
import { ErrorBoundary } from './components/ErrorBoundary';
import { Command } from './types';
import { TauriAPI } from './tauri';
import { NOTIFICATION_DURATION, RELOADED_MESSAGE } from './constants';
import { useAppEvents } from './hooks/useAppEvents';
import { usePreventHide } from './hooks/usePreventHide';
import { CommandProvider, useCommandContext } from './context/CommandContext';
import { SettingsProvider, useSettingsContext } from './context/SettingsContext';
import { daisyTheme } from './theme';
import './App.css';

import { NotificationProvider, useNotificationContext } from './context/NotificationContext';

// form は command があれば編集、template があれば複製、どちらも無ければ新規追加
type View =
    | { name: 'search' }
    | { name: 'form'; command?: Command; template?: Command }
    | { name: 'settings' };

// 内部コンポーネント: Contextを使用するために分離する
const AppContent = () => {
    const { showSuccess, showError, showWarning, reportError } = useNotificationContext();

    const {
        loading,
        error: commandsError,
        setError,
        loadCommands,
        addCommand,
        updateCommand,
        deleteCommand,
        clearUsage,
    } = useCommandContext();

    const { theme, fuzzyThreshold, maxResults, usageHistory, frequentFirst, loadSettings } =
        useSettingsContext();

    const [view, setView] = useState<View>({ name: 'search' });
    const [deletingCommand, setDeletingCommand] = useState<Command | undefined>(undefined);
    const [aliasingCommand, setAliasingCommand] = useState<Command | undefined>(undefined);

    usePreventHide(view.name !== 'search');

    const handleAddCommand = useCallback(() => setView({ name: 'form' }), []);

    const handleOpenSettings = useCallback(() => setView({ name: 'settings' }), []);

    const handleSaveCommand = useCallback(
        async (command: Command) => {
            const editing = view.name === 'form' && Boolean(view.command);
            const result = editing ? await updateCommand(command) : await addCommand(command);
            if (!result.ok) {
                showError(
                    `Could not ${editing ? 'update' : 'add'} "${command.name}": ${result.message}`,
                    NOTIFICATION_DURATION.LONG,
                );
                return;
            }
            showSuccess(`"${command.name}" ${editing ? 'updated' : 'added'} successfully.`);
            setView({ name: 'search' });
        },
        [view, updateCommand, addCommand, showSuccess, showError],
    );

    const handleConfirmDelete = useCallback(async () => {
        if (!deletingCommand) return;

        const result = await deleteCommand(deletingCommand.id, deletingCommand.name);
        if (!result.ok) {
            showError(
                `Could not delete "${deletingCommand.name}": ${result.message}`,
                NOTIFICATION_DURATION.LONG,
            );
            return;
        }
        showSuccess(`"${deletingCommand.name}" deleted successfully.`);
    }, [deletingCommand, deleteCommand, showSuccess, showError]);

    // The confirm dialog closes after either answer, and closing is what lands here
    const handleCancelDelete = useCallback(() => setDeletingCommand(undefined), []);

    const handleCancelForm = useCallback(() => setView({ name: 'search' }), []);

    const handleEditCommandFromSearch = useCallback(
        (command: Command) => setView({ name: 'form', command }),
        [],
    );

    const handleDuplicateCommandFromSearch = useCallback(
        (template: Command) => setView({ name: 'form', template }),
        [],
    );

    const handleIgnoreCommand = useCallback(
        async (command: Command) => {
            try {
                // The backend announces settings-reloaded, which refetches the list
                await TauriAPI.ignoreCommand(command.command);
                showSuccess(`Ignored "${command.name}"`, NOTIFICATION_DURATION.UNDO, {
                    label: 'Undo',
                    onClick: async () => {
                        try {
                            await TauriAPI.unignoreCommand(command.command);
                        } catch (err) {
                            reportError(
                                `undo ignoring "${command.name}"`,
                                err,
                                NOTIFICATION_DURATION.LONG,
                            );
                        }
                    },
                });
            } catch (err) {
                reportError(`ignore "${command.name}"`, err, NOTIFICATION_DURATION.LONG);
            }
        },
        [showSuccess, reportError],
    );

    const handleCancelAlias = useCallback(() => setAliasingCommand(undefined), []);

    // A rejection stays in the dialog, which shows the reason beside the field
    const handleSubmitAlias = useCallback(
        async (alias: string | null) => {
            if (!aliasingCommand) return;
            // The backend announces settings-reloaded, which refetches the list
            await TauriAPI.setExternalAlias(aliasingCommand.command, alias);
            showSuccess(
                alias
                    ? `Alias "${alias}" set for "${aliasingCommand.name}"`
                    : `Alias removed from "${aliasingCommand.name}"`,
            );
            setAliasingCommand(undefined);
        },
        [aliasingCommand, showSuccess],
    );

    const handleCopyCommand = useCallback(
        async (command: Command) => {
            if (!command.command) return;
            try {
                await navigator.clipboard.writeText(command.command);
                showSuccess('Copied to clipboard', 2000);
            } catch (err) {
                console.error('Failed to copy:', err);
                showError('Failed to copy');
            }
        },
        [showSuccess, showError],
    );

    const handleReloadCommands = useCallback(async () => {
        try {
            await TauriAPI.reloadAll(); // emits settings-reloaded, which refetches the list
            showSuccess(RELOADED_MESSAGE, 2000);
        } catch (err) {
            reportError('reload configuration', err);
        }
    }, [showSuccess, reportError]);

    // 初期化とイベントリスナー設定
    useAppEvents({
        handleAddCommand,
        handleOpenSettings,
        loadCommands,
        loadSettings,
        showWarning,
        showError,
        setError,
    });

    if (loading) {
        return (
            <div className="h-screen w-screen flex items-center justify-center bg-base-100 text-base-content">
                <p className="text-sm text-muted">Loading Scoot...</p>
            </div>
        );
    }

    if (commandsError) {
        return (
            <div
                role="alert"
                className="h-screen w-screen flex flex-col items-center justify-center gap-3 bg-base-100 text-base-content p-6 text-center"
            >
                <p className="text-sm">{commandsError}</p>
                <button className="btn btn-sm btn-primary" onClick={loadCommands}>
                    Retry
                </button>
            </div>
        );
    }

    return (
        <div className="app w-screen h-screen overflow-hidden" data-theme={daisyTheme(theme)}>
            {view.name === 'search' ? (
                <SearchWindow
                    fuzzyThreshold={fuzzyThreshold}
                    maxResults={maxResults}
                    usageHistory={usageHistory}
                    frequentFirst={frequentFirst}
                    onEditCommand={handleEditCommandFromSearch}
                    onDuplicateCommand={handleDuplicateCommandFromSearch}
                    onDeleteCommand={setDeletingCommand}
                    onAddCommand={handleAddCommand}
                    onCopyCommand={handleCopyCommand}
                    onAliasCommand={setAliasingCommand}
                    onIgnoreCommand={handleIgnoreCommand}
                    onReloadCommands={handleReloadCommands}
                    onOpenSettings={handleOpenSettings}
                    isDialogOpen={Boolean(deletingCommand || aliasingCommand)}
                />
            ) : view.name === 'settings' ? (
                <SettingsScreen
                    onClearHistory={clearUsage}
                    onBack={() => {
                        setView({ name: 'search' });
                        loadSettings();
                    }}
                />
            ) : (
                <CommandForm
                    command={view.command}
                    template={view.template}
                    onSave={handleSaveCommand}
                    onCancel={handleCancelForm}
                />
            )}

            <AliasDialog
                command={aliasingCommand}
                onSubmit={handleSubmitAlias}
                onCancel={handleCancelAlias}
            />

            <ConfirmDialog
                isOpen={Boolean(deletingCommand)}
                title="Delete Command"
                confirmLabel="Delete"
                destructive
                onConfirm={handleConfirmDelete}
                onCancel={handleCancelDelete}
            >
                Are you sure you want to delete{' '}
                <span className="font-semibold">{deletingCommand?.name}</span>?
                <br />
                This action cannot be undone.
            </ConfirmDialog>
        </div>
    );
};

function App() {
    return (
        <ErrorBoundary>
            <SettingsProvider>
                <NotificationProvider>
                    <CommandProvider>
                        <AppContent />
                    </CommandProvider>
                </NotificationProvider>
            </SettingsProvider>
        </ErrorBoundary>
    );
}

export default App;
