import React from 'react';
import { LuEllipsisVertical, LuSearch, LuChevronRight } from "react-icons/lu";
import { Command } from '../types';

interface SearchBarProps {
    query: string;
    onQueryChange: (query: string) => void;
    onKeyDown: (e: React.KeyboardEvent) => void;
    inputRef: React.RefObject<HTMLInputElement | null>;
    promptMode: { prompt: string; command: Command } | null;

    // Menu Actions
    onAddCommand: (e: React.MouseEvent) => void;
    onReloadCommands: (e: React.MouseEvent) => void;
    onOpenCommandsJson: (e: React.MouseEvent) => void;
    onOpenConfigJson: (e: React.MouseEvent) => void;
    onShowReadme: (e: React.MouseEvent) => void;
    onShowAbout: (e: React.MouseEvent) => void;
}

export const SearchBar: React.FC<SearchBarProps> = ({
    query,
    onQueryChange,
    onKeyDown,
    inputRef,
    promptMode,
    onAddCommand,
    onReloadCommands,
    onOpenCommandsJson,
    onOpenConfigJson,
    onShowReadme,
    onShowAbout,
}) => {
    return (
        <div className="flex-shrink-0 mb-4 relative">
            <div className="flex items-center gap-2">
                <label className="input input-sm input-bordered flex items-center gap-2 flex-1">
                    {promptMode ? (
                        <LuChevronRight />
                    ) : (
                        <LuSearch />
                    )}
                    <input
                        ref={inputRef}
                        type="text"
                        value={query}
                        onChange={(e) => onQueryChange(e.target.value)}
                        onKeyDown={onKeyDown}
                        placeholder={
                            promptMode
                                ? `${promptMode.command.name} - ${promptMode.command.description}`
                                : "Type to query..."
                        }
                        className="grow"
                        autoFocus
                    />
                </label>

                {/* ヘッダー3点ボタンとメニュー */}
                <div className="dropdown dropdown-end">
                    <div tabIndex={0} role="button" className="btn btn-ghost btn-sm btn-square p-2">
                        <LuEllipsisVertical />
                    </div>
                    <ul tabIndex={0} className="dropdown-content menu menu-sm bg-base-100 rounded-lg z-[1] w-48 p-2 shadow">
                        <li>
                            <a onClick={onAddCommand}>Add Command</a>
                        </li>
                        <li>
                            <a onClick={onReloadCommands}>Reload</a>
                        </li>
                        <li></li>
                        <li>
                            <a onClick={onOpenCommandsJson}>Open commands.json</a>
                        </li>
                        <li>
                            <a onClick={onOpenConfigJson}>Open config.json</a>
                        </li>
                        <li></li>
                        <li>
                            <a onClick={onShowReadme}>Show README</a>
                        </li>
                        <li>
                            <a onClick={onShowAbout}>About</a>
                        </li>
                    </ul>
                </div>
            </div>
        </div>
    );
};
