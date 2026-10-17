import React, { createContext, useContext, ReactNode } from 'react';
import { useCommands } from '../hooks/useCommands';
import { Command, UsageMap } from '../types';
import type { CommandResult } from '../hooks/useCommands';

interface CommandContextType {
    commands: Command[];
    usage: UsageMap;
    loading: boolean;
    error: string | null;
    setError: (error: string | null) => void;
    loadCommands: () => Promise<void>;
    addCommand: (command: Command) => Promise<CommandResult>;
    updateCommand: (command: Command) => Promise<CommandResult>;
    deleteCommand: (id: string, name: string) => Promise<CommandResult>;
    clearUsage: () => Promise<void>;
    executeCommand: (command: Command, args?: string[]) => Promise<boolean>; // rejects with the reason on failure
}

const CommandContext = createContext<CommandContextType | undefined>(undefined);

interface CommandProviderProps {
    children: ReactNode;
}

export const CommandProvider: React.FC<CommandProviderProps> = ({ children }) => {
    const commandLogic = useCommands();

    return <CommandContext.Provider value={commandLogic}>{children}</CommandContext.Provider>;
};

export const useCommandContext = () => {
    const context = useContext(CommandContext);
    if (context === undefined) {
        throw new Error('useCommandContext must be used within a CommandProvider');
    }
    return context;
};
