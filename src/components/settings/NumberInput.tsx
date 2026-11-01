import React, { useState } from 'react';

interface NumberInputProps {
    id?: string;
    /** 名前を示すラベルがないとき(表のセルなど)に使う */
    'aria-label'?: string;
    /** 大きさと幅。既定は設定行の右端に置くもの */
    className?: string;
    value: number;
    min: number;
    max: number;
    onChange: (value: number) => void;
}

/**
 * A whole-number field. It can be emptied and retyped (the stored value keeps its last number
 * meanwhile), every number typed is passed on within min and max, and leaving the field shows
 * the stored value.
 */
export const NumberInput: React.FC<NumberInputProps> = ({
    id,
    'aria-label': ariaLabel,
    className = 'input-sm w-24',
    value,
    min,
    max,
    onChange,
}) => {
    const [draft, setDraft] = useState<string | null>(null);

    return (
        <input
            id={id}
            aria-label={ariaLabel}
            type="number"
            min={min}
            max={max}
            className={`input ${className}`}
            value={draft ?? String(value)}
            onChange={(e) => {
                setDraft(e.target.value);
                const typed = Number.parseInt(e.target.value, 10);
                if (!Number.isNaN(typed)) onChange(Math.max(min, Math.min(max, typed)));
            }}
            onBlur={() => setDraft(null)}
        />
    );
};
