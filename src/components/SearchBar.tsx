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
                <label className="input input-sm flex items-center gap-2 flex-1 focus-within:outline-none">
                    {argumentMode ? <LuChevronRight /> : <LuSearch />}
                    <input
                        ref={inputRef}
                        type="text"
                        autoComplete="off"
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
                <div
                    className="dropdown dropdown-end"
                    onKeyDown={(e) => {
                        // Esc closes the menu: handing focus back to the search field collapses it
                        if (e.key === 'Escape') {
                            e.preventDefault();
                            e.stopPropagation();
                            inputRef.current?.focus();
                            return;
                        }
                        // Items are not tab stops (this is a search box first); arrows move through them
                        const keys = ['ArrowDown', 'ArrowUp', 'Home', 'End'];
                        if (!keys.includes(e.key)) return;
                        const items = Array.from(
                            e.currentTarget.querySelectorAll<HTMLElement>('[role="menuitem"]'),
                        );
                        const at = items.indexOf(document.activeElement as HTMLElement);
                        const next =
                            e.key === 'Home'
                                ? 0
                                : e.key === 'End'
                                  ? items.length - 1
                                  : e.key === 'ArrowDown'
                                    ? (at + 1) % items.length
                                    : at <= 0
                                      ? items.length - 1
                                      : at - 1;
                        e.preventDefault();
                        items[next]?.focus();
                    }}
                >
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
                        role="menu"
                        tabIndex={-1}
                        // Picking an item hands focus back to the search field, which also closes the menu
                        onClickCapture={() => inputRef.current?.focus()}
                        className="dropdown-content menu menu-sm bg-base-100 rounded-lg z-20 w-48 p-2 shadow-sm"
                    >
                        <li role="none">
                            <button
                                type="button"
                                role="menuitem"
                                tabIndex={-1}
                                onClick={onAddCommand}
                            >
                                Add Command
                            </button>
                        </li>
                        <li role="none">
                            <button
                                type="button"
                                role="menuitem"
                                tabIndex={-1}
                                onClick={(e) => {
                                    e.stopPropagation();
                                    onOpenConfig?.();
                                }}
                            >
                                Settings
                            </button>
                        </li>
                        <li role="none">
                            <button
                                type="button"
                                role="menuitem"
                                tabIndex={-1}
                                onClick={onReloadCommands}
                            >
                                Reload
                            </button>
                        </li>
                        <li
                            role="separator"
                            aria-hidden="true"
                            className="my-1 h-px bg-base-content/10 p-0"
                        ></li>
                        <li role="none">
                            <button
                                type="button"
                                role="menuitem"
                                tabIndex={-1}
                                onClick={onOpenCommandsJson}
                            >
                                Open commands.json
                            </button>
                        </li>
                        <li role="none">
                            <button
                                type="button"
                                role="menuitem"
                                tabIndex={-1}
                                onClick={onOpenConfigJson}
                            >
                                Open config.json
                            </button>
                        </li>
                        <li
                            role="separator"
                            aria-hidden="true"
                            className="my-1 h-px bg-base-content/10 p-0"
                        ></li>
                        <li role="none">
                            <button
                                type="button"
                                role="menuitem"
                                tabIndex={-1}
                                onClick={onShowReadme}
                            >
                                Show README
                            </button>
                        </li>
                        <li role="none">
                            <button
                                type="button"
                                role="menuitem"
                                tabIndex={-1}
                                onClick={onShowAbout}
                            >
                                About
                            </button>
                        </li>
                    </ul>
                </div>
            </div>
        </div>
    );
};
