/**
 * Refrescar lo de Clickedu solo (H11, `docs/01`).
 *
 * - `useRefrescoClickedu`, en `App`: pide `refrescar_clickedu` al abrir,
 *   cada minuto mientras la ventana se ve y al volver a ella. **El backend
 *   decide si toca** (encendido, los minutos desde el último intento, la
 *   cuenta): con la ventana minimizada no se pide nada.
 * - `useEstadoRefresco`: cuándo fue el último, si falló y lo que hay por
 *   traer, para Inicio, Conexión y Ajustes. Se pone al día con
 *   `refresco-estado`.
 * - Tras un refresco con datos nuevos llega `clickedu-refrescado`: las
 *   pantallas vuelven a leer lo de Clickedu sin perder lo que se hace.
 */

import {invoke} from '@tauri-apps/api/core';
import {listen} from '@tauri-apps/api/event';
import {useEffect, useState} from 'react';

import {t} from '@modules/i18n';

/** Lo leído de Clickedu ha cambiado (`commands::refresco::EVENTO`). */
export const EVENTO_REFRESCADO = 'clickedu-refrescado';
/** El estado del refresco ha cambiado (`commands::refresco::EVENTO_ESTADO`). */
export const EVENTO_ESTADO_REFRESCO = 'refresco-estado';

/** Cada cuánto se pregunta al backend si toca. */
const CADA_MS = 60_000;

/** Una plantilla que ha cambiado en Clickedu (`clickedu::refrescar::PorTraer`). */
export interface PorTraer {
  fichero: string;
  que: 'nueva' | 'cambia' | 'ya_no_esta';
}

/** `commands::refresco::EstadoRefresco`. */
export interface EstadoRefresco {
  encendido: boolean;
  minutos: number;
  /** Segundos Unix. */
  ultimo: number | null;
  fallo: string | null;
  /** Clickedu no acepta la cuenta: parado hasta volver a entrar. */
  parado: boolean;
  refrescando: boolean;
  porTraer: PorTraer[];
  avisos: string[];
}

/** Los minutos que se pueden elegir en Ajustes (`ajustes::Refresco::MINUTOS`). */
export const MINUTOS = [15, 30, 60] as const;

export function useRefrescoClickedu() {
  useEffect(() => {
    const pedir = () => {
      if (document.visibilityState === 'visible') invoke('refrescar_clickedu').catch(() => {});
    };
    pedir();
    const reloj = setInterval(pedir, CADA_MS);
    document.addEventListener('visibilitychange', pedir);
    return () => {
      clearInterval(reloj);
      document.removeEventListener('visibilitychange', pedir);
    };
  }, []);
}

/**
 * El estado (`undefined` mientras llega la primera vez) y con qué ponerlo
 * a mano: lo que devuelve `poner_refresco`, sin esperar al evento.
 */
export function useEstadoRefresco() {
  const [estado, setEstado] = useState<EstadoRefresco>();
  useEffect(() => {
    let vivo = true;
    const leer = () =>
      invoke<EstadoRefresco>('estado_refresco').then(
        (e) => vivo && setEstado(e),
        () => {},
      );
    leer();
    const quitar = listen(EVENTO_ESTADO_REFRESCO, leer);
    return () => {
      vivo = false;
      quitar.then((f) => f());
    };
  }, []);
  return [estado, setEstado] as const;
}

/**
 * Cuándo, para «Al día con Clickedu …»: «a las 9:40» si es de hoy y, si no,
 * «el 24/11/2026». La hora y no «hace 5 minutos», que se quedaría viejo con
 * la pantalla abierta.
 */
export function cuando(segundos: number, ahora = Date.now()): string {
  const d = new Date(segundos * 1000);
  return new Date(ahora).toDateString() === d.toDateString()
    ? t('refresco.a_las', {hora: d.toLocaleTimeString('es-ES', {hour: 'numeric', minute: '2-digit'})})
    : t('refresco.el_dia', {dia: d.toLocaleDateString('es-ES')});
}
