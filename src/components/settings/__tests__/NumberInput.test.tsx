import { useState } from 'react';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import '@testing-library/jest-dom';
import { NumberInput } from '../NumberInput';

const Field = ({ onChange = jest.fn() }: { onChange?: (n: number) => void }) => {
    const [value, setValue] = useState(10);
    return (
        <>
            <NumberInput
                id="n"
                value={value}
                min={1}
                max={100}
                onChange={(n) => {
                    setValue(n);
                    onChange(n);
                }}
            />
            <button>elsewhere</button>
            <output>{value}</output>
        </>
    );
};

describe('NumberInput', () => {
    it('can be emptied and retyped without jumping to the minimum', async () => {
        render(<Field />);
        const input = screen.getByRole('spinbutton');
        await userEvent.clear(input);
        expect(input).toHaveValue(null); // empty, not 1
        await userEvent.type(input, '25');
        expect(input).toHaveValue(25);
        expect(screen.getByRole('status')).toHaveTextContent('25');
    });

    it('passes on each number within the range', async () => {
        const onChange = jest.fn();
        render(<Field onChange={onChange} />);
        const input = screen.getByRole('spinbutton');
        await userEvent.clear(input);
        await userEvent.type(input, '250');
        expect(onChange).toHaveBeenLastCalledWith(100); // 250 is held to the maximum
        await userEvent.clear(input);
        await userEvent.type(input, '0');
        expect(onChange).toHaveBeenLastCalledWith(1); // 0 to the minimum
    });

    it('shows the stored value again after leaving the field empty', async () => {
        render(<Field />);
        const input = screen.getByRole('spinbutton');
        await userEvent.clear(input);
        await userEvent.click(screen.getByRole('button', { name: 'elsewhere' }));
        expect(input).toHaveValue(10);
    });

    it('shows the held value, not what was typed, once the field is left', async () => {
        render(<Field />);
        const input = screen.getByRole('spinbutton');
        await userEvent.clear(input);
        await userEvent.type(input, '250');
        await userEvent.click(screen.getByRole('button', { name: 'elsewhere' }));
        expect(input).toHaveValue(100);
    });
});
