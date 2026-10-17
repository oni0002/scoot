import { render } from '@testing-library/react';
import { createRef } from 'react';
import { useRestoreFocus } from '../useRestoreFocus';

const Harness = ({
    inputRef,
    restore,
}: {
    inputRef: React.RefObject<HTMLInputElement | null>;
    restore: boolean;
}) => {
    useRestoreFocus(inputRef, restore);
    return <input ref={inputRef} />;
};

describe('useRestoreFocus', () => {
    afterEach(() => jest.restoreAllMocks());

    it('focuses the input when focus was left on the body', () => {
        jest.spyOn(document, 'hasFocus').mockReturnValue(true);
        const ref = createRef<HTMLInputElement>();
        render(<Harness inputRef={ref} restore />);
        expect(document.activeElement).toBe(ref.current);
    });

    it('does nothing while the window is not focused, so click-away can still hide it', () => {
        jest.spyOn(document, 'hasFocus').mockReturnValue(false);
        const ref = createRef<HTMLInputElement>();
        render(<Harness inputRef={ref} restore />);
        expect(document.activeElement).toBe(document.body);
    });

    it('does nothing when restoring is not wanted', () => {
        jest.spyOn(document, 'hasFocus').mockReturnValue(true);
        const ref = createRef<HTMLInputElement>();
        render(<Harness inputRef={ref} restore={false} />);
        expect(document.activeElement).toBe(document.body);
    });
});
