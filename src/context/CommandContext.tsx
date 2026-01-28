import React, { createContext, useContext, ReactNode } from 'react';
import { useCommands } from '../hooks/useCommands';
import { Command } from '../types';

interface CommandContextType {
    commands: Command[];
    loading: boolean;
    error: string | null;
    setError: (error: string | null) => void;
    loadCommands: () => Promise<void>;
    addCommand: (command: Command) => Promise<boolean>;
    updateCommand: (command: Command) => Promise<boolean>;
    deleteCommand: (id: string, name: string) => Promise<boolean>;
    executeCommand: (command: Command, args?: string[]) => Promise<boolean>;
}

const CommandContext = createContext<CommandContextType | undefined>(undefined);

interface CommandProviderProps {
    children: ReactNode;
}

export const CommandProvider: React.FC<CommandProviderProps> = ({ children }) => {
    const commandLogic = useCommands();

    return (
        <CommandContext.Provider value={commandLogic}>
            {children}
        </CommandContext.Provider>
    );
};

export const useCommandContext = () => {
    const context = useContext(CommandContext);
    if (context === undefined) {
        throw new Error('useCommandContext must be used within a CommandProvider');
    }
    return context;
};
