/**
 * Los tres botones de la app en un componente — primary / stop / ghost.
 *
 * COPIADO de la Quality Test App. El estilo vive en
 * `@styles/buttons.module.css`; quien necesite una colocación puntual sigue
 * pudiendo pasar `style`.
 */

import {forwardRef} from 'react';
import styles from '@styles/buttons.module.css';

type Variante = 'primary' | 'stop' | 'ghost';

export interface ButtonProps
  extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: Variante;
}

const CLASE: Record<Variante, string> = {
  primary: styles.primary,
  stop: styles.stop,
  ghost: styles.ghost,
};

export const Button = forwardRef<HTMLButtonElement, ButtonProps>(function Button(
  {variant = 'primary', className, type = 'button', children, ...resto},
  ref,
) {
  const cls = [CLASE[variant], className].filter(Boolean).join(' ');
  return (
    <button ref={ref} type={type} className={cls} {...resto}>
      {children}
    </button>
  );
});
