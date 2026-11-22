/**
 * Las plantillas de Clickedu de la carpeta, una por clase (H1): su nombre
 * legible, el fichero, cuántos alumnos y cuántos tienen nota. Pulsar una la
 * abre en Calificar.
 *
 * La lista se pone al día sola: si se añade, cambia o quita un Excel de la
 * carpeta, el backend avisa (`plantillas-cambiadas`).
 */

import {invoke} from '@tauri-apps/api/core';

import {Punto} from '@components/clases/Avisos';
import {BarraAvance} from '@components/clases/BarraAvance';
import {Ruta, SinPlantillas} from '@components/clases/SinPlantillas';
import {Cabecera} from '@components/layout/Cabecera';
import {Button} from '@components/ui/Button';
import {Icon, ICONOS} from '@components/ui/Icon';
import type {Clase} from '@modules/clases/tipos';
import {useClases} from '@modules/clases/useClases';
import {t} from '@modules/i18n';
import {useIdioma} from '@modules/i18n/useIdioma';

/** La última, la `›` que dice que la fila se pulsa. */
const COLUMNAS = 'minmax(220px, 1fr) 96px 120px minmax(120px, 0.6fr) 20px';

export interface ClasesProps {
  abrir: (fichero: string) => void;
  irAAjustes: () => void;
}

export function Clases({abrir, irAAjustes}: ClasesProps) {
  useIdioma();
  const {lista, error} = useClases({contarNotas: true});
  const hay = (lista?.clases.length ?? 0) > 0;

  return (
    <div style={{padding: '28px 32px'}}>
      <Cabecera
        titulo={t('clases.titulo')}
        intro={t('clases.intro')}
        derecha={
          lista?.carpeta.existe && (
            <Button variant="ghost" onClick={() => invoke('abrir_carpeta_plantillas').catch(() => {})}>
              <Icon d={ICONOS.carpeta} size={14} />
              {t('clases.abrir_carpeta')}
            </Button>
          )
        }
      />
      {error && (
        <p role="alert" style={{margin: '0 0 16px', fontSize: 13, color: 'var(--danger)'}}>
          {error}
        </p>
      )}
      {lista && !hay && <SinPlantillas carpeta={lista.carpeta} irAAjustes={irAAjustes} />}
      {lista && hay && (
        <>
          <div style={{marginTop: -8, marginBottom: 14}}>
            <Ruta ruta={lista.carpeta.ruta} />
          </div>
          <Tabla clases={lista.clases} abrir={abrir} />
        </>
      )}
    </div>
  );
}

/** Las clases; las pendientes de otros cursos, aparte y al final, como en Conexión. */
function Tabla({clases, abrir}: {clases: Clase[]; abrir: (fichero: string) => void}) {
  const normales = clases.filter((c) => !c.pendiente);
  const pendientes = clases.filter((c) => c.pendiente);
  return (
    <div style={{border: '1px solid var(--border)', borderRadius: 8, overflow: 'hidden', background: 'var(--bg-card)'}}>
      <Encabezado>
        <span>{t('clases.col_clase')}</span>
        <span style={{textAlign: 'right'}}>{t('clases.col_alumnos')}</span>
        <span style={{textAlign: 'right'}}>{t('clases.col_calificados')}</span>
        <span style={{paddingLeft: 24}}>{t('clases.col_estado')}</span>
      </Encabezado>
      {normales.map((c, i) => (
        <Fila key={c.fichero} c={c} abrir={abrir} ultima={i === normales.length - 1 && pendientes.length === 0} />
      ))}
      {pendientes.length > 0 && (
        <>
          <Encabezado arriba={normales.length > 0}>
            <span>{t('clases.pendientes')}</span>
          </Encabezado>
          {pendientes.map((c, i) => (
            <Fila key={c.fichero} c={c} abrir={abrir} ultima={i === pendientes.length - 1} />
          ))}
        </>
      )}
    </div>
  );
}

function Encabezado({children, arriba}: {children: React.ReactNode; arriba?: boolean}) {
  return (
    <div
      style={{
        display: 'grid',
        gridTemplateColumns: COLUMNAS,
        alignItems: 'center',
        height: 32,
        padding: '0 var(--row-pad)',
        background: 'var(--bg-sunk)',
        borderTop: arriba ? '1px solid var(--border)' : undefined,
        borderBottom: '1px solid var(--border)',
        fontSize: 10.5,
        fontWeight: 600,
        letterSpacing: 0.8,
        textTransform: 'uppercase',
        color: 'var(--text-tertiary)',
      }}>
      {children}
    </div>
  );
}

function Fila({c, abrir, ultima}: {c: Clase; abrir: (fichero: string) => void; ultima: boolean}) {
  return (
    <button
      type="button"
      onClick={() => abrir(c.fichero)}
      className="fila-clase"
      style={{
        display: 'grid',
        gridTemplateColumns: COLUMNAS,
        alignItems: 'center',
        width: '100%',
        minHeight: 48,
        padding: '6px var(--row-pad)',
        border: 'none',
        borderBottom: ultima ? 'none' : '1px solid var(--border)',
        background: 'transparent',
        textAlign: 'left',
        font: 'inherit',
        cursor: 'pointer',
      }}>
      <span style={{minWidth: 0}}>
        <span style={{display: 'block', fontSize: 13.5, color: 'var(--text-primary)', ...cortar}}>{c.nombre}</span>
        <span
          style={{
            display: 'block',
            fontFamily: 'var(--font-mono)',
            fontSize: 11,
            color: 'var(--text-tertiary)',
            ...cortar,
          }}
          title={c.fichero}>
          {c.fichero}
        </span>
      </span>
      {c.lectura.tipo === 'bien' ? (
        <>
          <span style={cifra}>{c.lectura.alumnos}</span>
          <span
            role="img"
            aria-label={t('clases.con_nota', {n: c.lectura.calificados, total: c.lectura.alumnos})}
            title={t('clases.con_nota', {n: c.lectura.calificados, total: c.lectura.alumnos})}
            style={{display: 'flex', justifyContent: 'flex-end'}}>
            <BarraAvance fraccion={c.lectura.alumnos > 0 ? c.lectura.calificados / c.lectura.alumnos : 0} ancho={72} />
          </span>
          <span
            style={{
              paddingLeft: 24,
              fontSize: 12,
              color: 'var(--text-secondary)',
              display: 'flex',
              alignItems: 'center',
              gap: 8,
            }}>
            {c.lectura.avisos > 0 && (
              <>
                <Punto />
                {c.lectura.avisos === 1 ? t('clases.un_aviso') : t('clases.n_avisos', {n: c.lectura.avisos})}
              </>
            )}
          </span>
        </>
      ) : (
        <span
          style={{gridColumn: '2 / 5', textAlign: 'right', fontSize: 12, color: 'var(--danger)', ...cortar}}
          title={c.lectura.error}>
          {t('clases.no_se_lee', {error: c.lectura.error})}
        </span>
      )}
      <Icon d={ICONOS.galonDerecha} size={14} style={{justifySelf: 'end', color: 'var(--text-tertiary)'}} />
    </button>
  );
}

const cortar: React.CSSProperties = {overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap'};
const cifra: React.CSSProperties = {
  textAlign: 'right',
  fontFamily: 'var(--font-mono)',
  fontSize: 13,
  color: 'var(--text-primary)',
};
