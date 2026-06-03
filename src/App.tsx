import { useState, useCallback } from 'react';
import { SearchWindow } from './components/SearchWindow';
import { CommandForm } from './components/CommandForm';
import { DeleteConfirmDialog } from './components/DeleteConfirmDialog';
import { ErrorBoundary } from './components/ErrorBoundary';
import { Command } from './types';
import { TauriAPI } from './tauri';
import { NOTIFICATION_DURATION } from './constants';
import { useAppEvents } from './hooks/useAppEvents';
import { CommandProvider, useCommandContext } from './context/CommandContext';
import { ConfigProvider, useConfigContext } from './context/ConfigContext';
import './App.css';

import { NotificationProvider, useNotificationContext } from './context/NotificationContext';

// 内部コンポーネント: Contextを使用するために分離する
const AppContent = () => {
    const {
        showSuccess,
        showError,
        showWarning,
        showInfo,
        // removeNotification is handled by Provider internally now
    } = useNotificationContext();

    const {
        loading,
        error: commandsError,
        setError,
        loadCommands,
        addCommand,
        updateCommand,
        deleteCommand,
    } = useCommandContext();

    const { theme, fuzzyThreshold, maxResults, loadConfig } = useConfigContext();

    const [currentView, setCurrentView] = useState<'search' | 'form'>('search');
    const [editingCommand, setEditingCommand] = useState<Command | undefined>(undefined);
    const [showDeleteDialog, setShowDeleteDialog] = useState(false);
    const [deletingCommand, setDeletingCommand] = useState<Command | undefined>(undefined);

    const handleAddCommand = useCallback(() => {
        setEditingCommand(undefined);
        setCurrentView('form');
    }, []);

    const handleSaveCommand = useCallback(
        async (command: Command) => {
            let success: boolean;
            if (editingCommand) {
                success = await updateCommand(command);
                if (success) showSuccess(`"${command.name}" updated successfully.`);
                else showError(`Failed to update "${command.name}".`, NOTIFICATION_DURATION.LONG);
            } else {
                success = await addCommand(command);
                if (success) showSuccess(`"${command.name}" added successfully.`);
                else showError(`Failed to add "${command.name}".`, NOTIFICATION_DURATION.LONG);
            }

            if (success) {
                setCurrentView('search');
                setEditingCommand(undefined);
            }
        },
        [editingCommand, updateCommand, addCommand, showSuccess, showError],
    );

    const handleConfirmDelete = useCallback(
        async (command?: Command) => {
            const targetCommand = command || deletingCommand;
            if (!targetCommand) return;

            const success = await deleteCommand(targetCommand.id, targetCommand.name);
            if (success) {
                showSuccess(`"${targetCommand.name}" deleted successfully.`);
                setShowDeleteDialog(false);
                setDeletingCommand(undefined);
            } else {
                showError(`Could not delete "${targetCommand.name}".`, NOTIFICATION_DURATION.LONG);
            }
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
        }
    }, [showDeleteDialog]);

    const handleEditCommandFromSearch = useCallback((command: Command) => {
        setEditingCommand(command);
        setCurrentView('form');
    }, []);

    const handleDeleteCommandFromSearch = useCallback((command: Command) => {
        setDeletingCommand(command);
        setShowDeleteDialog(true);
    }, []);

    const handleIgnoreCommand = useCallback(
        async (command: Command) => {
            try {
                const config = await TauriAPI.getConfig();
                if (!config.ignored.includes(command.command)) {
                    const updated = { ...config, ignored: [...config.ignored, command.command] };
                    await TauriAPI.saveConfig(updated);
                    await TauriAPI.reloadAll();
                }
                await loadCommands();
                showSuccess(`"${command.name}" ignored successfully.`);
            } catch (err) {
                console.error('Failed to ignore command:', err);
                showError(`Could not ignore "${command.name}".`, NOTIFICATION_DURATION.LONG);
            }
        },
        [loadCommands, showSuccess, showError],
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
            await TauriAPI.reloadAll();
            await loadCommands();
            // showSuccess("Commands and config reloaded", 2000); // Handled by event listener
        } catch (err) {
            console.error('Failed to reload:', err);
            showError('Could not reload configuration');
        }
    }, [loadCommands, showError]);

    // 初期化とイベントリスナー設定
    useAppEvents({
        handleAddCommand,
        loadCommands,
        loadConfig,
        showSuccess,
        showWarning,
        showInfo,
        showError,
        setError,
    });

    if (loading) {
        return (
            <div className="app-loading">
                <p>Loading Scoot...</p>
            </div>
        );
    }

    if (commandsError) {
        return (
            <div className="app-error">
                <p>Error: {commandsError}</p>
                <button onClick={loadCommands}>Retry</button>
            </div>
        );
    }

    return (
        <div className="app w-screen h-screen overflow-hidden" data-theme={theme}>
            {currentView === 'search' ? (
                <SearchWindow
                    fuzzyThreshold={fuzzyThreshold}
                    maxResults={maxResults}
                    onEditCommand={handleEditCommandFromSearch}
                    onDeleteCommand={handleDeleteCommandFromSearch}
                    onAddCommand={handleAddCommand}
                    onCopyCommand={handleCopyCommand}
                    onIgnoreCommand={handleIgnoreCommand}
                    onReloadCommands={handleReloadCommands}
                    isDialogOpen={showDeleteDialog}
                />
            ) : (
                <CommandForm
                    command={editingCommand}
                    onSave={handleSaveCommand}
                    onCancel={handleCancelDialog}
                />
            )}

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
        <ErrorBoundary
            onError={(error, errorInfo) => {
                console.error('React Error Boundary caught error:', error, errorInfo);
                alert('An unexpected error occurred. Please try restarting the application.');
            }}
        >
            <NotificationProvider>
                <ConfigProvider>
                    <CommandProvider>
                        <AppContent />
                    </CommandProvider>
                </ConfigProvider>
            </NotificationProvider>
        </ErrorBoundary>
    );
}

export default App;
