import { useState, useCallback } from "react";
import { SearchWindow } from "./components/SearchWindow";
import { CommandForm } from "./components/CommandForm";
import { DeleteConfirmDialog } from "./components/DeleteConfirmDialog";
import { ErrorBoundary } from "./components/ErrorBoundary";
import { TauriAPI } from "./api/tauri";
import { Command } from "./types";
import { useAppEvents } from "./hooks/useAppEvents";
import { CommandProvider, useCommandContext } from "./context/CommandContext";
import "./App.css";

import { NotificationProvider, useNotificationContext } from "./context/NotificationContext";

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
    ignoreCommand,
  } = useCommandContext();

  const [currentView, setCurrentView] = useState<'search' | 'form'>('search');
  const [editingCommand, setEditingCommand] = useState<Command | undefined>(undefined);
  const [showDeleteDialog, setShowDeleteDialog] = useState(false);
  const [deletingCommand, setDeletingCommand] = useState<Command | undefined>(undefined);
  const [theme, setTheme] = useState<string>("dark");
  const [fuzzyThreshold, setFuzzyThreshold] = useState<number>(0.5);
  const [maxResults, setMaxResults] = useState<number>(10);

  const loadConfig = useCallback(async () => {
    try {
      const config = await TauriAPI.getConfig();
      setTheme(config.theme || "dark");
      setFuzzyThreshold(config.fuzzyThreshold || 0.5);
      setMaxResults(config.maxResults || 10);
    } catch (err) {
      console.warn("Failed to load config, using default values:", err);
      setTheme("dark");
      setFuzzyThreshold(0.5);
      setMaxResults(10);
    }
  }, []);

  const handleAddCommand = useCallback(async () => {
    console.log('App: handleAddCommand called');
    await TauriAPI.setPreventHide(true);
    setEditingCommand(undefined);
    setCurrentView('form');
  }, []);

  const handleSaveCommand = useCallback(async (command: Command) => {
    console.log('App: handleSaveCommand called', command);

    let success = false;
    if (editingCommand) {
      success = await updateCommand(command);
    } else {
      success = await addCommand(command);
    }

    if (success) {
      setCurrentView('search');
      setEditingCommand(undefined);
    }
  }, [editingCommand, updateCommand, addCommand]);

  const handleConfirmDelete = useCallback(async (command?: Command) => {
    const targetCommand = command || deletingCommand;
    if (!targetCommand) return;

    const success = await deleteCommand(targetCommand.id, targetCommand.name);
    if (success) {
      setShowDeleteDialog(false);
      setDeletingCommand(undefined);
    }
  }, [deletingCommand, deleteCommand]);

  const handleCancelDialog = useCallback(() => {
    if (showDeleteDialog) {
      setShowDeleteDialog(false);
      setDeletingCommand(undefined);
    } else {
      setCurrentView('search');
      setEditingCommand(undefined);
    }
  }, [showDeleteDialog]);

  const handleEditCommandFromSearch = useCallback(async (command: Command) => {
    console.log('App: handleEditCommandFromSearch called', command);
    await TauriAPI.setPreventHide(true);
    setEditingCommand(command);
    setCurrentView('form');
  }, []);

  const handleDeleteCommandFromSearch = useCallback((command: Command) => {
    setDeletingCommand(command);
    setShowDeleteDialog(true);
  }, []);

  const handleCopyCommand = useCallback(async (command: Command) => {
    if (!command.command) return;
    try {
      await navigator.clipboard.writeText(command.command);
      showSuccess("Copied to clipboard", 2000);
    } catch (err) {
      console.error("Failed to copy:", err);
      showError("Failed to copy");
    }
  }, [showSuccess, showError]);

  const handleReloadCommands = useCallback(async () => {
    try {
      await TauriAPI.reloadAll();
      await loadCommands();
      // showSuccess("Commands and config reloaded", 2000); // Handled by event listener
    } catch (err) {
      console.error("Failed to reload:", err);
      showError("Could not reload configuration");
    }
  }, [loadCommands, showSuccess, showError]);

  // 初期化とイベントリスナー設定
  useAppEvents({
    handleAddCommand,
    loadCommands,
    loadConfig,
    showSuccess,
    showWarning,
    showInfo,
    showError,
    setError
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
          onIgnoreCommand={ignoreCommand}
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
        console.error("React Error Boundary caught error:", error, errorInfo);
        alert("An unexpected error occurred. Please try restarting the application.");
      }}
    >
      <NotificationProvider>
        <CommandProvider>
          <AppContent />
        </CommandProvider>
      </NotificationProvider>
    </ErrorBoundary>
  );
}

export default App;
