import { useState, useEffect, useCallback } from "react";
import { SearchWindow } from "./components/SearchWindow";
import { CommandForm } from "./components/CommandForm";
import { DeleteConfirmDialog } from "./components/DeleteConfirmDialog";
import { NotificationToast } from "./components/NotificationToast";
import { ErrorBoundary } from "./components/ErrorBoundary";
import { TauriAPI } from "./api/tauri";
import { Command } from "./types";
import { useNotifications } from "./hooks/useNotifications";
import { useAppEvents } from "./hooks/useAppEvents";
import { CommandProvider, useCommandContext } from "./context/CommandContext";
import { NOTIFICATION_DURATION } from "./constants";
import "./App.css";

// 内部コンポーネント: Contextを使用するために分離する
const AppContent = ({
  notifications,
  showSuccess,
  showError,
  showWarning,
  showInfo,
  removeNotification
}: {
  notifications: any[];
  showSuccess: (msg: string, duration?: number) => void;
  showError: (msg: string, duration?: number) => void;
  showWarning: (msg: string) => void;
  showInfo: (msg: string) => void;
  removeNotification: (id: string) => void;
}) => {
  const {
    loading,
    error: commandsError,
    setError,
    loadCommands,
    addCommand,
    updateCommand,
    deleteCommand,
  } = useCommandContext();

  const [currentView, setCurrentView] = useState<'search' | 'form'>('search');
  const [editingCommand, setEditingCommand] = useState<Command | undefined>(undefined);
  const [showDeleteDialog, setShowDeleteDialog] = useState(false);
  const [deletingCommand, setDeletingCommand] = useState<Command | undefined>(undefined);
  const [theme, setTheme] = useState<string>("dark");
  const [fuzzyThreshold, setFuzzyThreshold] = useState<number>(0.5);

  const loadConfig = useCallback(async () => {
    try {
      const config = await TauriAPI.getConfig();
      setTheme(config.theme || "dark");
      setFuzzyThreshold(config.fuzzy_threshold || 0.5);
    } catch (err) {
      console.warn("Failed to load config, using default values:", err);
      setTheme("dark");
      setFuzzyThreshold(0.5);
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
      await TauriAPI.reloadConfig();
      await loadCommands();
      showSuccess("Commands and config reloaded", 2000);
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

  // テーマ切り替え（開発用）
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.ctrlKey && e.shiftKey && e.key === 'T') {
        const themes = ["dark", "light", "dracula", "coffee"];
        const currentIndex = themes.indexOf(theme);
        const nextTheme = themes[(currentIndex + 1) % themes.length];
        setTheme(nextTheme);
      }
    };

    document.addEventListener('keydown', handleKeyDown);
    return () => document.removeEventListener('keydown', handleKeyDown);
  }, [theme]);

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
          onEditCommand={handleEditCommandFromSearch}
          onDeleteCommand={handleDeleteCommandFromSearch}
          onAddCommand={handleAddCommand}
          onCopyCommand={handleCopyCommand}
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

      <NotificationToast
        notifications={notifications}
        onRemove={removeNotification}
      />
    </div>
  );
};

function App() {
  const { notifications, removeNotification, showSuccess, showError, showWarning, showInfo } = useNotifications();

  return (
    <ErrorBoundary
      onError={(error, errorInfo) => {
        console.error("React Error Boundary caught error:", error, errorInfo);
        showError(
          "An unexpected error occurred. Please try restarting the application.",
          NOTIFICATION_DURATION.CRITICAL
        );
      }}
    >
      <CommandProvider showSuccess={showSuccess} showError={showError} showInfo={showInfo}>
        <AppContent
          notifications={notifications}
          showSuccess={showSuccess}
          showError={showError}
          showWarning={showWarning}
          showInfo={showInfo}
          removeNotification={removeNotification}
        />
      </CommandProvider>
    </ErrorBoundary>
  );
}

export default App;
