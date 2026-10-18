import { useState, useCallback } from 'react';
import { SearchWindow } from './components/SearchWindow';
import { CommandForm } from './components/CommandForm';
import { ConfigScreen } from './components/ConfigScreen';
import { DeleteConfirmDialog } from './components/DeleteConfirmDialog';
import { AliasDialog } from './components/AliasDialog';
import { ErrorBoundary } from './components/ErrorBoundary';
import { Command } from './types';
import { TauriAPI } from './tauri';
import { NOTIFICATION_DURATION } from './constants';
import { getErrorMessage } from './error';
import { useAppEvents } from './hooks/useAppEvents';
import { usePreventHide } from './hooks/usePreventHide';
import { CommandProvider, useCommandContext } from './context/CommandContext';
import { ConfigProvider, useConfigContext } from './context/ConfigContext';
import { daisyTheme } from './theme';
import './App.css';

import { NotificationProvider, useNotificationContext } from './context/NotificationContext';

// 内部コンポーネント: Contextを使用するために分離する
const AppContent = () => {
    const { showSuccess, showError, showWarning } = useNotificationContext();

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

    const { theme, fuzzyThreshold, maxResults, usageHistory, loadConfig } = useConfigContext();

    const [currentView, setCurrentView] = useState<'search' | 'form' | 'config'>('search');
    const [editingCommand, setEditingCommand] = useState<Command | undefined>(undefined);
    const [duplicatingCommand, setDuplicatingCommand] = useState<Command | undefined>(undefined);
    const [showDeleteDialog, setShowDeleteDialog] = useState(false);
    const [deletingCommand, setDeletingCommand] = useState<Command | undefined>(undefined);
    const [aliasingCommand, setAliasingCommand] = useState<Command | undefined>(undefined);

    usePreventHide(currentView !== 'search');

    const handleAddCommand = useCallback(() => {
        setEditingCommand(undefined);
        setDuplicatingCommand(undefined);
        setCurrentView('form');
    }, []);

    const handleOpenConfig = useCallback(() => {
        setCurrentView('config');
    }, []);

    const handleSaveCommand = useCallback(
        async (command: Command) => {
            const editing = Boolean(editingCommand);
            const result = editing ? await updateCommand(command) : await addCommand(command);
            if (!result.ok) {
                showError(
                    `Could not ${editing ? 'update' : 'add'} "${command.name}": ${result.message}`,
                    NOTIFICATION_DURATION.LONG,
                );
                return;
            }
            showSuccess(`"${command.name}" ${editing ? 'updated' : 'added'} successfully.`);
            setCurrentView('search');
            setEditingCommand(undefined);
            setDuplicatingCommand(undefined);
        },
        [editingCommand, updateCommand, addCommand, showSuccess, showError],
    );

    const handleConfirmDelete = useCallback(
        async (command?: Command) => {
            const targetCommand = command || deletingCommand;
            if (!targetCommand) return;

            const result = await deleteCommand(targetCommand.id, targetCommand.name);
            if (!result.ok) {
                showError(
                    `Could not delete "${targetCommand.name}": ${result.message}`,
                    NOTIFICATION_DURATION.LONG,
                );
                return;
            }
            showSuccess(`"${targetCommand.name}" deleted successfully.`);
            setShowDeleteDialog(false);
            setDeletingCommand(undefined);
        },
        [deletingCommand, deleteCommand, showSuccess, showError],
    );

    const handleCancelDialog = useCallback(() => {
        if (showDeleteDialog) {
            setShowDeleteDialog(false);
            setDeletingCommand(undefined);
        } else {
            setCurrentView('search');
            setEditingCommand(undefined);
            setDuplicatingCommand(undefined);
        }
    }, [showDeleteDialog]);

    const handleEditCommandFromSearch = useCallback((command: Command) => {
        setEditingCommand(command);
        setDuplicatingCommand(undefined);
        setCurrentView('form');
    }, []);

    const handleDuplicateCommandFromSearch = useCallback((command: Command) => {
        setEditingCommand(undefined);
        setDuplicatingCommand(command);
        setCurrentView('form');
    }, []);

    const handleDeleteCommandFromSearch = useCallback((command: Command) => {
        setDeletingCommand(command);
        setShowDeleteDialog(true);
    }, []);

    const handleIgnoreCommand = useCallback(
        async (command: Command) => {
            try {
                // The backend announces config-reloaded, which refetches the list
                await TauriAPI.ignoreCommand(command.command);
                showSuccess(`Ignored "${command.name}"`, NOTIFICATION_DURATION.UNDO, {
                    label: 'Undo',
                    onClick: async () => {
                        try {
                            await TauriAPI.unignoreCommand(command.command);
                        } catch (err) {
                            showError(
                                `Could not undo ignoring "${command.name}": ${getErrorMessage(err)}`,
                                NOTIFICATION_DURATION.LONG,
                            );
                        }
                    },
                });
            } catch (err) {
                console.error('Failed to ignore command:', err);
                showError(
                    `Could not ignore "${command.name}": ${getErrorMessage(err)}`,
                    NOTIFICATION_DURATION.LONG,
                );
            }
        },
        [showSuccess, showError],
    );

    const handleCancelAlias = useCallback(() => setAliasingCommand(undefined), []);

    // A rejection stays in the dialog, which shows the reason beside the field
    const handleSubmitAlias = useCallback(
        async (alias: string | null) => {
            if (!aliasingCommand) return;
            // The backend announces config-reloaded, which refetches the list
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
            await TauriAPI.reloadAll(); // emits config-reloaded, which refetches the list
            showSuccess('Commands and config reloaded', 2000);
        } catch (err) {
            console.error('Failed to reload:', err);
            showError(`Could not reload configuration: ${getErrorMessage(err)}`);
        }
    }, [showSuccess, showError]);

    // 初期化とイベントリスナー設定
    useAppEvents({
        handleAddCommand,
        handleOpenConfig,
        loadCommands,
        loadConfig,
        showSuccess,
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
            {currentView === 'search' ? (
                <SearchWindow
                    fuzzyThreshold={fuzzyThreshold}
                    maxResults={maxResults}
                    usageHistory={usageHistory}
                    onEditCommand={handleEditCommandFromSearch}
                    onDuplicateCommand={handleDuplicateCommandFromSearch}
                    onDeleteCommand={handleDeleteCommandFromSearch}
                    onAddCommand={handleAddCommand}
                    onCopyCommand={handleCopyCommand}
                    onAliasCommand={setAliasingCommand}
                    onIgnoreCommand={handleIgnoreCommand}
                    onReloadCommands={handleReloadCommands}
                    onOpenConfig={handleOpenConfig}
                    isDialogOpen={showDeleteDialog || Boolean(aliasingCommand)}
                />
            ) : currentView === 'config' ? (
                <ConfigScreen
                    onClearHistory={clearUsage}
                    onBack={() => {
                        setCurrentView('search');
                        loadConfig();
                    }}
                />
            ) : (
                <CommandForm
                    command={editingCommand}
                    template={duplicatingCommand}
                    onSave={handleSaveCommand}
                    onCancel={handleCancelDialog}
                />
            )}

            <AliasDialog
                command={aliasingCommand}
                onSubmit={handleSubmitAlias}
                onCancel={handleCancelAlias}
            />

            <DeleteConfirmDialog
                isOpen={showDeleteDialog}
                command={deletingCommand}
                onConfirm={handleConfirmDelete}
                onCancel={handleCancelDialog}
            />
        </div>
    );
};

function App() {
    return (
        <ErrorBoundary>
            <ConfigProvider>
                <NotificationProvider>
                    <CommandProvider>
                        <AppContent />
                    </CommandProvider>
                </NotificationProvider>
            </ConfigProvider>
        </ErrorBoundary>
    );
}

export default App;
