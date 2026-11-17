/**
 * `<select>` nativo con el estilo del resto de campos.
 *
 * COPIADO de library-app (y esta, de la Quality Test App). Para listas
 * largas —una clase de 30 alumnos, las asignaturas de Clickedu— el que sirve es
 * `Dropdown`, que abre por portal y no depende del desplegable del sistema.
 */

import {forwardRef} from 'react';
import styles from '@styles/inputs.module.css';

export interface SelectOption {
  value: string;
  label: string;
  disabled?: boolean;
}

export interface SelectProps
  extends React.SelectHTMLAttributes<HTMLSelectElement> {
  options?: SelectOption[];
}

export const Select = forwardRef<HTMLSelectElement, SelectProps>(function Select(
  {className, options, children, ...resto},
  ref,
) {
  return (
    <select
      ref={ref}
      className={[styles.select, className].filter(Boolean).join(' ')}
      {...resto}>
      {options
        ? options.map((o) => (
            <option key={o.value} value={o.value} disabled={o.disabled}>
              {o.label}
            </option>
          ))
        : children}
    </select>
  );
});
