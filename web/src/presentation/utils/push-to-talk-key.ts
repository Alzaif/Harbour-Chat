const PRESET_KEYS = ['Space', 'KeyV', 'KeyB', 'KeyC', 'KeyT'] as const;

/** Reserved for cancelling key capture, so it can't be bound as a push-to-talk key. */
const DISALLOWED_KEYS = new Set(['Escape']);

const MODIFIER_LABELS: Record<string, string> = {
  ShiftLeft: 'Left Shift',
  ShiftRight: 'Right Shift',
  ControlLeft: 'Left Ctrl',
  ControlRight: 'Right Ctrl',
  AltLeft: 'Left Alt',
  AltRight: 'Right Alt',
  MetaLeft: 'Left Meta',
  MetaRight: 'Right Meta',
};

const NAMED_LABELS: Record<string, string> = {
  Backquote: '`',
  Minus: '-',
  Equal: '=',
  BracketLeft: '[',
  BracketRight: ']',
  Backslash: '\\',
  Semicolon: ';',
  Quote: "'",
  Comma: ',',
  Period: '.',
  Slash: '/',
  Tab: 'Tab',
  Enter: 'Enter',
  Backspace: 'Backspace',
  CapsLock: 'Caps Lock',
  Insert: 'Insert',
  Delete: 'Delete',
  Home: 'Home',
  End: 'End',
  PageUp: 'Page Up',
  PageDown: 'Page Down',
};

export function formatKeyCode(code: string): string {
  if (!code) return '';
  if (code === 'Space') return 'Space';
  if (code.startsWith('Key')) return code.slice(3);
  if (code.startsWith('Digit')) return code.slice(5);
  if (code.startsWith('Numpad')) return `Numpad ${code.slice(6)}`.trim();
  if (code.startsWith('Arrow')) return `${code.slice(5)} Arrow`;
  if (MODIFIER_LABELS[code]) return MODIFIER_LABELS[code]!;
  if (NAMED_LABELS[code]) return NAMED_LABELS[code]!;
  return code;
}

export function isAllowedPushToTalkKey(code: string): boolean {
  if (!code) return false;
  return !DISALLOWED_KEYS.has(code);
}

export { PRESET_KEYS };
