/**
 * Campo de texto y campo numérico, que comparten estilo.
 *
 * COPIADO de la Quality Test App. Los dos propagan la ref y aceptan todos los
 * atributos nativos, así que funcionan dentro de un formulario sin tocarlos.
 */

import {forwardRef} from 'react';
import styles from '@styles/inputs.module.css';

export interface TextInputProps
  extends React.InputHTMLAttributes<HTMLInputElement> {}

export const TextInput = forwardRef<HTMLInputElement, TextInputProps>(
  function TextInput({className, type = 'text', ...resto}, ref) {
    return (
      <input
        ref={ref}
        type={type}
        className={[styles.text, className].filter(Boolean).join(' ')}
        {...resto}
      />
    );
  },
);

export interface NumInputProps extends Omit<TextInputProps, 'type'> {
  /** Unidad que se pinta al lado: `cm`, `€`, `págs`. */
  unit?: string;
}

/**
 * Campo numérico sin la ruleta del sistema.
 *
 * No lleva −/+ propio: `<input type="number">` ya sube y baja con **↑ y ↓ del
 * teclado**, que es lo único que hacía la ruleta. Lo que sí se quita es que la
 * **rueda del ratón** cambie el valor, que pasaba con solo posar el cursor
 * encima al hacer scroll.
 */
export const NumInput = forwardRef<HTMLInputElement, NumInputProps>(
  function NumInput({className, unit, ...resto}, ref) {
    return (
      <div className={styles.unitWrap}>
        <input
          ref={ref}
          type="number"
          className={[styles.num, className].filter(Boolean).join(' ')}
          onWheel={(e) => e.currentTarget.blur()}
          {...resto}
        />
        {unit && <span className={styles.unit}>{unit}</span>}
      </div>
    );
  },
);
