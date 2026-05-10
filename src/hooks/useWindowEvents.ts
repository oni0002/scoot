import { useClickOutsideHide } from './useClickOutsideHide';
import { useAltSpaceHide } from './useAltSpaceHide';
import { useWindowShownListener } from './useWindowShownListener';
import { useInputFocus } from './useInputFocus';

export const useWindowEvents = (
  isDialogOpen: boolean,
  resetState: () => void,
  inputRef: React.RefObject<HTMLInputElement | null>
) => {
  useInputFocus(inputRef);
  useClickOutsideHide(isDialogOpen);
  useAltSpaceHide(isDialogOpen);
  useWindowShownListener(resetState);
};
