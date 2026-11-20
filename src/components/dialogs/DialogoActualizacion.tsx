/**
 * El diálogo de una versión nueva (H7): qué trae, descargarla e instalarla.
 *
 * Es el `DialogoActualizacion` de library-app sobre el actualizador oficial
 * de Tauri, con la carcasa de los diálogos de esta app:
 *
 *   aviso → descargando → lista → (instalar: la app se cierra)
 *                    ↘ error ↙
 *
 * - **La firma se comprueba al descargar** y otra vez al instalar. Si no
 *   casa, se dice y no se instala nada.
 * - Nunca instala sin que se pulse; «Saltar esta versión» hace que no se
 *   ofrezca al arrancar (a mano, sí).
 * - Las insignias de «Cambios» llevan color, como en library-app: una
 *   versión nueva **es un evento** (`docs/02` §3). Nunca rojo, que no es un
 *   error (D6). El ámbar, oscurecido: `--warn` como texto no se lee.
 */

import {useEffect, useState} from 'react';

import {Modal} from '@components/dialogs/Modal';
import {Button} from '@components/ui/Button';
import {Progreso} from '@components/ui/Progreso';
import {
  type AvanceDescarga,
  descargarActualizacion,
  instalarActualizacion,
  type VersionNueva,
} from '@modules/actualizacion';
import {t} from '@modules/i18n';
import type {TranslationKey} from '@modules/i18n/es';

/** El color de cada tipo: el del texto y, al 14 %, el del fondo. */
const COLOR = {
  feat: 'var(--success)',
  fix: 'var(--accent-ink)',
  perf: 'color-mix(in oklch, var(--warn) 65%, black)',
  chore: 'var(--text-tertiary)',
} as const;
type Tipo = keyof typeof COLOR;
const insignia = (tipo: string) => {
  const t2 = (tipo in COLOR ? tipo : 'chore') as Tipo;
  return {rotulo: `actualizacion.tipo_${t2}` as TranslationKey, color: COLOR[t2]};
};

const FECHA = new Intl.DateTimeFormat('es-ES', {day: 'numeric', month: 'long', year: 'numeric'});
/** En MB enteros, como la descarga del modelo de voz. */
const aMb = (bytes: number) => Math.round(bytes / 1e6);

function fecha(dia: string) {
  if (!dia) return '';
  try {
    return FECHA.format(new Date(`${dia}T12:00:00`));
  } catch {
    return dia;
  }
}

type Fase =
  | {fase: 'aviso'}
  | {fase: 'descargando'; avance: AvanceDescarga}
  | {fase: 'lista'}
  | {fase: 'instalando'}
  | {fase: 'error'; titulo: string; mensaje: string; descargada: boolean};

export interface DialogoActualizacionProps {
  nueva: VersionNueva | null;
  cerrar: () => void;
  saltar: (version: string) => void;
}

export function DialogoActualizacion({nueva, cerrar, saltar}: DialogoActualizacionProps) {
  const [f, setF] = useState<Fase>({fase: 'aviso'});
  useEffect(() => setF({fase: 'aviso'}), [nueva]);

  if (!nueva) return null;
  const v = nueva;
  const ocupado = f.fase === 'descargando' || f.fase === 'instalando';

  const descargar = async () => {
    setF({fase: 'descargando', avance: {hecho: 0, total: 0}});
    try {
      await descargarActualizacion((avance) => setF((x) => (x.fase === 'descargando' ? {...x, avance} : x)));
      setF({fase: 'lista'});
    } catch (e) {
      setF({fase: 'error', titulo: t('actualizacion.error_descarga'), mensaje: String(e), descargada: false});
    }
  };

  const instalar = async () => {
    setF({fase: 'instalando'});
    try {
      await instalarActualizacion();
      // Si ha salido bien, la app ya se está cerrando.
    } catch (e) {
      setF({fase: 'error', titulo: t('actualizacion.error_instalar'), mensaje: String(e), descargada: true});
    }
  };

  const publicada = fecha(v.publicada);

  return (
    <Modal open onClose={ocupado ? undefined : cerrar} width={500} ariaLabel={t('actualizacion.etiqueta')}>
      <div style={{padding: '20px 22px 14px'}}>
        <div style={{fontSize: 12, fontWeight: 600, color: 'var(--accent-ink)'}}>{t('actualizacion.etiqueta')}</div>
        <div style={{display: 'flex', alignItems: 'baseline', justifyContent: 'space-between', gap: 8, marginTop: 4}}>
          <span style={{fontSize: 26, fontWeight: 600, fontFamily: 'var(--font-display)', letterSpacing: '-0.02em'}}>
            v{v.version}
          </span>
          <span style={{fontSize: 12, color: 'var(--text-tertiary)', fontFamily: 'var(--font-mono)'}}>
            {t('actualizacion.actual', {version: v.actual})}
          </span>
        </div>
        {publicada && (
          <div style={{marginTop: 2, fontSize: 12, color: 'var(--text-tertiary)'}}>
            {t('actualizacion.publicada', {fecha: publicada})}
          </div>
        )}
      </div>

      <div style={{overflowY: 'auto', minHeight: 0, padding: '0 22px'}}>
        {v.titular && (
          <p
            style={{
              margin: 0,
              padding: '10px 12px',
              borderRadius: 8,
              background: 'var(--bg-sunk)',
              fontSize: 13,
              lineHeight: 1.55,
              color: 'var(--text-secondary)',
            }}>
            {v.titular}
          </p>
        )}
        {v.entradas.length > 0 && (
          <ul style={{listStyle: 'none', margin: '14px 0 0', padding: 0, display: 'grid', gap: 7}}>
            {v.entradas.map((e, i) => {
              const {rotulo, color} = insignia(e.tipo);
              return (
                <li key={i} style={{display: 'flex', alignItems: 'flex-start', gap: 10}}>
                  <span
                    style={{
                      flexShrink: 0,
                      // Lo que mide «Más rápido», la más larga: el texto, alineado.
                      minWidth: 76,
                      boxSizing: 'border-box',
                      marginTop: 2,
                      padding: '1px 6px',
                      borderRadius: 4,
                      background: `color-mix(in oklch, ${color} 14%, transparent)`,
                      fontSize: 10.5,
                      fontWeight: 600,
                      textAlign: 'center',
                      color,
                    }}>
                    {t(rotulo)}
                  </span>
                  <span style={{fontSize: 13, lineHeight: 1.5, color: 'var(--text-secondary)'}}>{e.texto}</span>
                </li>
              );
            })}
          </ul>
        )}
      </div>

      <div style={{padding: '14px 22px 0', minHeight: 0}}>
        {f.fase === 'descargando' && (
          <Progreso rotulo={t('actualizacion.descargando')} hecho={aMb(f.avance.hecho)} total={aMb(f.avance.total)} />
        )}
        {f.fase === 'lista' && (
          <p style={{margin: 0, fontSize: 13, lineHeight: 1.5, color: 'var(--text-secondary)'}}>
            {t('actualizacion.lista')}
          </p>
        )}
        {f.fase === 'instalando' && (
          <p style={{margin: 0, fontSize: 13, color: 'var(--text-secondary)'}}>{t('actualizacion.instalando')}</p>
        )}
        {f.fase === 'error' && (
          <div role="alert" style={{fontSize: 13, lineHeight: 1.5}}>
            <div style={{fontWeight: 600, color: 'var(--danger)'}}>{f.titulo}</div>
            <div style={{color: 'var(--text-secondary)', overflowWrap: 'anywhere'}}>{f.mensaje}</div>
          </div>
        )}
      </div>

      <div
        style={{
          display: 'flex',
          justifyContent: 'space-between',
          alignItems: 'center',
          gap: 8,
          marginTop: 14,
          padding: '12px 22px',
          borderTop: '1px solid var(--border)',
          background: 'var(--bg-sunk)',
        }}>
        <Button variant="ghost" onClick={() => saltar(v.version)} disabled={ocupado}>
          {t('actualizacion.saltar')}
        </Button>
        <div style={{display: 'flex', gap: 8}}>
          {!ocupado && (
            <Button variant="ghost" onClick={cerrar}>
              {t('actualizacion.mas_tarde')}
            </Button>
          )}
          {f.fase === 'aviso' && <Button onClick={descargar}>{t('actualizacion.descargar')}</Button>}
          {ocupado && (
            <Button disabled>
              {t(f.fase === 'descargando' ? 'actualizacion.descargando_boton' : 'actualizacion.instalando_boton')}
            </Button>
          )}
          {f.fase === 'lista' && <Button onClick={instalar}>{t('actualizacion.instalar')}</Button>}
          {f.fase === 'error' && (
            <Button onClick={f.descargada ? instalar : descargar}>{t('actualizacion.reintentar')}</Button>
          )}
        </div>
      </div>
    </Modal>
  );
}
