import { createPortal } from 'react-dom';
import type { ReactNode } from 'react';

export function ModalPortal({ children }: { children: ReactNode }) {
  if (typeof document === 'undefined') return children;
  return createPortal(children, document.body);
}
