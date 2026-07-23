import { useEffect } from 'react';

export function usePushToTalk(options: {
  enabled: boolean;
  active: boolean;
  keyCode: string;
  onTransmitChange: (transmitting: boolean) => void;
}) {
  const { enabled, active, keyCode, onTransmitChange } = options;

  useEffect(() => {
    if (!enabled || !active) return;

    const setTransmitting = (value: boolean) => {
      onTransmitChange(value);
    };

    const onKeyDown = (event: KeyboardEvent) => {
      if (event.code !== keyCode || event.repeat) return;
      if (event.target instanceof HTMLInputElement || event.target instanceof HTMLTextAreaElement) {
        return;
      }
      event.preventDefault();
      setTransmitting(true);
    };

    const onKeyUp = (event: KeyboardEvent) => {
      if (event.code !== keyCode) return;
      setTransmitting(false);
    };

    const onBlur = () => setTransmitting(false);

    window.addEventListener('keydown', onKeyDown);
    window.addEventListener('keyup', onKeyUp);
    window.addEventListener('blur', onBlur);
    return () => {
      window.removeEventListener('keydown', onKeyDown);
      window.removeEventListener('keyup', onKeyUp);
      window.removeEventListener('blur', onBlur);
      setTransmitting(false);
    };
  }, [enabled, active, keyCode, onTransmitChange]);
}
