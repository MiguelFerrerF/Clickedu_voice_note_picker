/**
 * La carcasa de los diálogos: centrado, fondo atenuado, Esc para cerrar.
 *
 * COPIADO de la Quality Test App, salvo que Esc se oye en captura. Los hijos
 * ponen su propio relleno y cabecera: `Modal` es solo la colocación y el
 * fondo.
 */

import {useEffect} from 'react';

export interface ModalProps {
  open: boolean;
  /** Sin él, ni Esc ni el fondo cierran: para cuando hay algo en marcha. */
  onClose?: () => void;
  width?: number | string;
  maxHeight?: number | string;
  closeOnBackdrop?: boolean;
  children?: React.ReactNode;
  ariaLabel?: string;
}

export function Modal({
  open,
  onClose,
  width = 560,
  maxHeight = '88vh',
  closeOnBackdrop = true,
  children,
  ariaLabel,
}: ModalProps) {
  useEffect(() => {
    if (!open) return;
    // En captura y parado, como el `Dropdown`: debajo puede haber un campo
    // en edición, y allí Esc cancela. Con el modal abierto, Esc es del modal.
    const onKey = (e: KeyboardEvent) => {
      if (e.key !== 'Escape') return;
      e.stopPropagation();
      onClose?.();
    };
    window.addEventListener('keydown', onKey, true);
    const prev = document.body.style.overflow;
    document.body.style.overflow = 'hidden';
    return () => {
      window.removeEventListener('keydown', onKey, true);
      document.body.style.overflow = prev;
    };
  }, [open, onClose]);

  if (!open) return null;

  return (
    <div
      role="dialog"
      aria-modal="true"
      aria-label={ariaLabel}
      style={{
        position: 'fixed',
        inset: 0,
        zIndex: 1200,
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        background: 'oklch(0 0 0 / 0.45)',
        animation: 'fadeIn 140ms ease-out',
        padding: 24,
      }}
      onClick={(e) => {
        if (closeOnBackdrop && e.target === e.currentTarget) onClose?.();
      }}>
      <div
        style={{
          background: 'var(--bg-card)',
          border: '1px solid var(--border)',
          borderRadius: 14,
          width,
          maxWidth: '100%',
          maxHeight,
          overflow: 'hidden',
          display: 'flex',
          flexDirection: 'column',
          boxShadow: '0 24px 64px oklch(0 0 0 / 0.25), 0 0 0 1px oklch(1 0 0 / 0.02)',
          animation: 'slideUp 180ms cubic-bezier(.3,.7,.4,1)',
        }}>
        {children}
      </div>
    </div>
  );
}
