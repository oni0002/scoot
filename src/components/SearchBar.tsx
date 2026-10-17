import React from 'react';
import { LuEllipsisVertical, LuSearch, LuChevronRight } from 'react-icons/lu';
import { Command } from '../types';
import { RESULT_LIST_ID } from '../constants';

interface SearchBarProps {
    query: string;
    onQueryChange: (query: string) => void;
    onKeyDown: (e: React.KeyboardEvent) => void;
    inputRef: React.RefObject<HTMLInputElement | null>;
    argumentMode: { alias: string; command: Command } | null;

    // Menu Actions
    onAddCommand: (e: React.MouseEvent) => void;
    onReloadCommands: (e: React.MouseEvent) => void;
    onOpenCommandsJson: (e: React.MouseEvent) => void;
    onOpenConfigJson: (e: React.MouseEvent) => void;
    onShowReadme: (e: React.MouseEvent) => void;
    onShowAbout: (e: React.MouseEvent) => void;
    onOpenConfig?: () => void;

    // Combobox wiring: the input drives the result listbox without taking focus away from it
    hasResults: boolean;
    activeDescendantId?: string;
}

export const SearchBar: React.FC<SearchBarProps> = ({
    query,
    onQueryChange,
    onKeyDown,
    inputRef,
    argumentMode,
    onAddCommand,
    onReloadCommands,
    onOpenCommandsJson,
    onOpenConfigJson,
    onShowReadme,
    onShowAbout,
    onOpenConfig,
    hasResults,
    activeDescendantId,
}) => {
    return (
        <div className="shrink-0 mb-4 relative">
            <div className="flex items-center gap-4">
                <label className="input input-sm flex items-center gap-2 flex-1">
                    {argumentMode ? <LuChevronRight /> : <LuSearch />}
                    <input
                        ref={inputRef}
                        type="text"
                        value={query}
                        onChange={(e) => onQueryChange(e.target.value)}
                        onKeyDown={onKeyDown}
                        placeholder={
                            argumentMode
                                ? `${argumentMode.command.name} - ${argumentMode.command.description}`
                                : 'Where do you wanna scoot?'
                        }
                        className="grow"
                        autoFocus
                        role="combobox"
                        aria-label="Search commands"
                        aria-autocomplete="list"
                        aria-expanded={hasResults}
                        aria-controls={RESULT_LIST_ID}
                        aria-activedescendant={activeDescendantId}
                    />
                </label>

                {/* Dropdown menu */}
                <div className="dropdown dropdown-end">
                    <div
                        tabIndex={0}
                        role="button"
                        aria-label="More actions"
                        aria-haspopup="menu"
                        className="btn btn-ghost btn-sm btn-square p-2"
                    >
                        <LuEllipsisVertical />
                    </div>
                    <ul
                        tabIndex={0}
                        className="dropdown-content menu menu-sm bg-base-100 rounded-lg z-[1] w-48 p-2 shadow-sm"
                    >
                        <li>
                            <a onClick={onAddCommand}>Add Command</a>
                        </li>
                        <li>
                            <a
                                onClick={(e) => {
                                    e.stopPropagation();
                                    onOpenConfig?.();
                                }}
                            >
                                Settings
                            </a>
                        </li>
                        <li>
                            <a onClick={onReloadCommands}>Reload</a>
                        </li>
                        <li
                            role="separator"
                            aria-hidden="true"
                            className="my-1 h-px bg-base-content/10 p-0"
                        ></li>
                        <li>
                            <a onClick={onOpenCommandsJson}>Open commands.json</a>
                        </li>
                        <li>
                            <a onClick={onOpenConfigJson}>Open config.json</a>
                        </li>
                        <li
                            role="separator"
                            aria-hidden="true"
                            className="my-1 h-px bg-base-content/10 p-0"
                        ></li>
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
