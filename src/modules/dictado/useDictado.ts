/**
 * El push-to-talk (H3a): **mantener ESPACIO** mientras se dice el alumno y
 * la nota, y soltarlo.
 *
 * - **El micrófono** se abre mientras Calificar está a la vista, con una
 *   clase abierta y la ventana delante (`activo`), y se cierra en cuanto no:
 *   así Windows solo enseña que se usa cuando se puede dictar. Abierto,
 *   guarda los últimos 0,3 s, para no perder el principio del nombre.
 * - **ESPACIO** se coge antes que nadie (en captura, en `window`): ni hace
 *   bajar la página ni pulsa el botón que tenga el foco. Vale también con el
 *   foco en una celda de nota, donde un espacio no sirve de nada. Lo que lo
 *   necesita de verdad se marca con `data-sin-dictado`.
 * - Una pulsación corta (menos de `CORTO_MS`) no se manda a Whisper.
 * - Si la ventana deja de estar delante con ESPACIO pulsado, el `keyup` no
 *   llega: lo grabado se tira.
 * - Mientras Whisper entiende una frase se puede ir diciendo la siguiente.
 * - Mientras se escucha, el backend manda el **nivel** del micrófono (de 0
 *   a 1, en decibelios) 20 veces por segundo, para el medidor (HUI).
 *
 * Las llamadas al micrófono (abrir, empezar, cerrar) van **en cola**: si
 * no, un cerrar podría adelantar al abrir y dejarlo abierto.
 *
 * El modelo de voz (181 MB) se descarga la primera vez, con avance.
 */

import {Channel, invoke} from '@tauri-apps/api/core';
import {useEffect, useRef, useState} from 'react';

import type {Dictado} from '@modules/dictado/tipos';

/** Por debajo, la pulsación es sin querer. */
const CORTO_MS = 300;

/** El modelo de voz: si está, o cómo va su descarga (en bytes). */
export type Modelo =
  | {tipo: 'comprobando'}
  | {tipo: 'falta'; fallo?: string}
  | {tipo: 'descargando'; hecho: number; total: number}
  | {tipo: 'listo'};

/** Lo que no es un dictado pero conviene decir. */
export type AvisoVoz = 'nada_oido' | 'corto';

export interface Dictando {
  modelo: Modelo;
  /** El nombre del micrófono abierto. */
  microfono?: string;
  escuchando: boolean;
  /** Cómo se oye mientras se escucha, de 0 a 1. */
  nivel: number;
  /** Cuántas frases está entendiendo Whisper. */
  entendiendo: number;
  aviso?: AvisoVoz;
  /** El micrófono o Whisper han fallado, con el motivo. */
  fallo?: {tipo: 'microfono' | 'dictado'; error: string};
  descargar: () => void;
}

export interface UseDictadoProps {
  /** Se puede dictar: la pantalla está a la vista y hay una clase abierta. */
  activo: boolean;
  /** La clase con la que se entiende lo dicho. */
  fichero: string | undefined;
  /** Lo entendido, con la clase con la que se entendió. */
  alDictar: (fichero: string, dictado: Dictado) => void;
}

export function useDictado({activo, fichero, alDictar}: UseDictadoProps): Dictando {
  const [modelo, setModelo] = useState<Modelo>({tipo: 'comprobando'});
  const [microfono, setMicrofono] = useState<string>();
  const [escuchando, setEscuchando] = useState(false);
  const [nivel, setNivel] = useState(0);
  const [entendiendo, setEntendiendo] = useState(0);
  const [aviso, setAviso] = useState<AvisoVoz>();
  const [fallo, setFallo] = useState<Dictando['fallo']>();
  const [delante, setDelante] = useState(() => document.hasFocus());

  // Lo último de cada render, para los manejadores de teclado y las promesas.
  const ficheroAhora = useRef(fichero);
  ficheroAhora.current = fichero;
  const alDictarAhora = useRef(alDictar);
  alDictarAhora.current = alDictar;

  const cola = useRef<Promise<unknown>>(Promise.resolve());
  const enCola = <T>(f: () => Promise<T>): Promise<T> => {
    const hecho = cola.current.then(f, f);
    cola.current = hecho.catch(() => {});
    return hecho;
  };
  // Cuándo se pulsó ESPACIO (0 = no está pulsado) y lo que hace falta para
  // terminar después de haber empezado.
  const pulsado = useRef(0);
  const empezado = useRef<Promise<unknown>>(Promise.resolve());

  useEffect(() => {
    invoke<boolean>('hay_modelo').then(
      (hay) => setModelo(hay ? {tipo: 'listo'} : {tipo: 'falta'}),
      (e) => setModelo({tipo: 'falta', fallo: String(e)}),
    );
  }, []);

  useEffect(() => {
    const alEntrar = () => setDelante(true);
    const alSalir = () => setDelante(false);
    window.addEventListener('focus', alEntrar);
    window.addEventListener('blur', alSalir);
    return () => {
      window.removeEventListener('focus', alEntrar);
      window.removeEventListener('blur', alSalir);
    };
  }, []);

  const abierto = activo && delante && modelo.tipo === 'listo';

  // El micrófono, abierto solo mientras se puede dictar.
  useEffect(() => {
    if (!abierto) return;
    setFallo(undefined);
    enCola(() => invoke<string>('abrir_microfono')).then(setMicrofono, (e) =>
      setFallo({tipo: 'microfono', error: String(e)}),
    );
    return () => {
      if (pulsado.current) {
        pulsado.current = 0;
        setEscuchando(false);
        enCola(() => invoke('cancelar_dictado')).catch(() => {});
      }
      setMicrofono(undefined);
      enCola(() => invoke('cerrar_microfono')).catch(() => {});
    };
  }, [abierto]);

  // ESPACIO.
  useEffect(() => {
    if (!abierto) return;
    const esEspacio = (e: KeyboardEvent) =>
      e.code === 'Space' &&
      !e.ctrlKey &&
      !e.altKey &&
      !e.metaKey &&
      !e.shiftKey &&
      !(e.target instanceof Element && e.target.closest('[data-sin-dictado]'));

    const abajo = (e: KeyboardEvent) => {
      if (!esEspacio(e)) return;
      e.preventDefault();
      e.stopPropagation();
      if (e.repeat || pulsado.current) return;
      pulsado.current = performance.now();
      setEscuchando(true);
      setNivel(0);
      setAviso(undefined);
      setFallo(undefined);
      // Lo que llegue después de soltar no se pinta.
      const canal = new Channel<number>();
      canal.onmessage = (n) => pulsado.current && setNivel(n);
      empezado.current = enCola(() => invoke('empezar_dictado', {nivel: canal}));
      empezado.current.catch((e) => setFallo({tipo: 'microfono', error: String(e)}));
    };

    const arriba = (e: KeyboardEvent) => {
      if (e.code !== 'Space' || !pulsado.current) return;
      e.preventDefault();
      e.stopPropagation();
      const duracion = performance.now() - pulsado.current;
      pulsado.current = 0;
      setEscuchando(false);
      if (duracion < CORTO_MS) {
        enCola(() => invoke('cancelar_dictado')).catch(() => {});
        setAviso('corto');
        return;
      }
      const f = ficheroAhora.current;
      if (!f) return;
      setEntendiendo((n) => n + 1);
      // Fuera de la cola: mientras Whisper entiende, se puede empezar otra.
      // Si no se pudo empezar, ya se ha dicho por qué.
      empezado.current
        .then(
          () =>
            invoke<Dictado | null>('terminar_dictado', {fichero: f}).then(
              (d) => (d ? alDictarAhora.current(f, d) : setAviso('nada_oido')),
              (e) => setFallo({tipo: 'dictado', error: String(e)}),
            ),
          () => {},
        )
        .finally(() => setEntendiendo((n) => n - 1));
    };

    window.addEventListener('keydown', abajo, true);
    window.addEventListener('keyup', arriba, true);
    return () => {
      window.removeEventListener('keydown', abajo, true);
      window.removeEventListener('keyup', arriba, true);
    };
  }, [abierto]);

  function descargar() {
    const avance = new Channel<{hecho: number; total: number}>();
    avance.onmessage = ({hecho, total}) => setModelo({tipo: 'descargando', hecho, total});
    setModelo({tipo: 'descargando', hecho: 0, total: 0});
    invoke('descargar_modelo', {avance}).then(
      () => setModelo({tipo: 'listo'}),
      (e) => setModelo({tipo: 'falta', fallo: String(e)}),
    );
  }

  return {modelo, microfono, escuchando, nivel, entendiendo, aviso, fallo, descargar};
}
