import React, { createContext, useContext, ReactNode } from 'react';
import { useCommands } from '../hooks/useCommands';

type CommandContextType = ReturnType<typeof useCommands>;

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
