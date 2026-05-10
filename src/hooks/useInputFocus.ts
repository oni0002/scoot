import { useEffect } from 'react';

export const useInputFocus = (inputRef: React.RefObject<HTMLInputElement | null>) => {
  useEffect(() => {
    inputRef.current?.focus();
  }, [inputRef]);
};
