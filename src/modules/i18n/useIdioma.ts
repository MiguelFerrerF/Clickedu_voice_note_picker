/**
 * `useIdioma` — hook que re-renderiza cuando cambia el idioma activo.
 *
 * Lo llama todo componente que use `t()` dentro del render. El valor que
 * devuelve se usa como disparador de render, no tanto por su contenido —
 * hoy solo puede ser `'es'`.
 */

import {useEffect, useState} from 'react';

import {BUS_I18N, getIdioma, type Idioma} from '.';

export function useIdioma(): Idioma {
  const [idioma, setEstado] = useState<Idioma>(getIdioma());

  useEffect(() => {
    const alCambiar = (e: Event) => {
      setEstado((e as CustomEvent<{idioma: Idioma}>).detail.idioma);
    };
    BUS_I18N.addEventListener('cambio-idioma', alCambiar);
    return () => BUS_I18N.removeEventListener('cambio-idioma', alCambiar);
  }, []);

  return idioma;
}
