import React, { useState } from 'react';
import { LuCheck, LuCopy } from 'react-icons/lu';
import { Command } from '../types';
import { expandTemplate, missingSlots, slotsOf, substituteArgs, unusedArgs } from '../command';

interface ArgumentPanelProps {
    command: Command;
    args: string[];
}

const SLOT_CLASS = 'rounded-sm bg-base-content/15 px-1';

const VERB: Record<string, string> = { url: 'Opens', file: 'Opens', command: 'Runs' };

const statusFor = (command: Command, args: string[]): string | null => {
    const missing = missingSlots(command.command, args);
    if (missing.length > 0) return `Waiting for ${missing.join(', ')}. Type it after the alias.`;

    const unused = unusedArgs(command.command, args);
    if (unused.length > 0) {
        const takes = slotsOf(command.command).join(', ');
        const quoted = unused.map((a) => `"${a}"`).join(', ');
        return `${quoted} ${unused.length > 1 ? 'are' : 'is'} not used: this command only takes ${takes || 'no arguments'}.`;
    }
    return null;
};

/** Shown instead of results once an alias is entered: what will run, and what is still missing. */
export const ArgumentPanel: React.FC<ArgumentPanelProps> = ({ command, args }) => {
    const [copied, setCopied] = useState(false);
    const urlEncode = command.category === 'url';
    const result = substituteArgs(command.command, args, urlEncode);
    const status = statusFor(command, args);
    const missing = missingSlots(command.command, args).length > 0;

    const copyResult = () => {
        navigator.clipboard.writeText(result);
        setCopied(true);
        setTimeout(() => setCopied(false), 3000);
    };

    return (
        <div className="space-y-4 pt-1">
            {/* One frame: the selected-row look, extended down to hold what the alias will do */}
            <div className="rounded-lg bg-base-content/10 ring-1 ring-inset ring-base-content/50">
                <div className="flex items-center gap-2 p-2">
                    {command.alias && (
                        <div className="badge badge-sm badge-primary">{command.alias}</div>
                    )}
                    <div className="font-medium text-sm truncate">{command.name}</div>
                    <div className="text-xs truncate flex-1 min-w-0">{command.description}</div>
                </div>

                <div className="px-2 pb-2">
                    <div className="flex items-center gap-2 rounded-md bg-base-100 py-1 pl-2 pr-1 text-xs">
                        {/* The verb only needs to be seen for shell commands; a URL or path reads as opening */}
                        <span className={command.category === 'command' ? 'text-muted' : 'sr-only'}>
                            {VERB[command.category] ?? 'Runs'}
                        </span>
                        <span className="min-w-0 flex-1 font-mono break-all">
                            {expandTemplate(command.command, args, urlEncode).map((part, i) =>
                                part.filled ? (
                                    <span key={i} className="font-bold text-accent">
                                        {part.text}
                                    </span>
                                ) : part.slot ? (
                                    <span key={i} className={SLOT_CLASS}>
                                        {part.text}
                                    </span>
                                ) : (
                                    <span key={i}>{part.text}</span>
                                ),
                            )}
                        </span>
                        <button
                            type="button"
                            onClick={copyResult}
                            className="btn btn-square btn-xs btn-ghost"
                            title="Copy"
                            aria-label="Copy result"
                        >
                            {copied ? <LuCheck /> : <LuCopy />}
                        </button>
                    </div>
                </div>
            </div>

            {/* A missing argument is already visible as the {$n} chip, so it is only announced; a dropped
                argument has no visual cue, so that one is written out. */}
            <p role="status" className={missing ? 'sr-only' : 'px-2 text-xs min-h-4'}>
                {status}
            </p>
        </div>
    );
};
