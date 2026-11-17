/**
 * Contenedor genérico. Título + subtítulo opcional + cuerpo.
 *
 * COPIADO de la Quality Test App. El hueco `right` es para acciones (un lápiz
 * de editar, un contador); así el título no se le echa encima cuando parte en
 * dos líneas.
 */

import styles from '@styles/card.module.css';

export interface CardProps {
  title?: React.ReactNode;
  subtitle?: React.ReactNode;
  right?: React.ReactNode;
  children?: React.ReactNode;
  style?: React.CSSProperties;
  className?: string;
}

export function Card({
  title,
  subtitle,
  right,
  children,
  style,
  className,
}: CardProps) {
  return (
    <div
      className={[styles.card, className].filter(Boolean).join(' ')}
      style={style}>
      {(title || subtitle || right) && (
        <div className={styles.header}>
          <div style={{minWidth: 0}}>
            {title && <h3 className={styles.title}>{title}</h3>}
            {subtitle && <div className={styles.subtitle}>{subtitle}</div>}
          </div>
          {right && <div style={{flexShrink: 0}}>{right}</div>}
        </div>
      )}
      <div className={styles.body}>{children}</div>
    </div>
  );
}
