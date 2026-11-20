/**
 * La versión anterior (H7): lo que se ha traído de la v1.3.4 al abrir, y el
 * botón para quitarla.
 *
 * Traer no pide nada (`traer_de_la_v1`, al arrancar); quitarla, sí: si se
 * instaló para todo el equipo, Windows pide permiso. Por eso es un botón y se
 * avisa antes. Con «Más tarde», al abrir la app otra vez se vuelve a ofrecer.
 *
 * Ninguna normalización en silencio: se dice cuántas clases y notas han
 * llegado y adónde, las que entraron con otro nombre para no pisar nada, y
 * si la carpeta de Ajustes ha cambiado.
 */

import {invoke} from '@tauri-apps/api/core';
import {useState} from 'react';

import {Modal} from '@components/dialogs/Modal';
import {Button} from '@components/ui/Button';
import {Progreso} from '@components/ui/Progreso';
import {t} from '@modules/i18n';

/** `commands::migracion::PasoV1`. */
export interface PasoV1 {
  traido: {clases: number; notas: number; ya_estaban: number; renombrados: string[]};
  plantillas: string;
  notas: string;
  carpeta_cambiada: boolean;
  pide_permiso: boolean;
}

type Fase = {fase: 'aviso'} | {fase: 'quitando'} | {fase: 'quitada'} | {fase: 'error'; mensaje: string};

const P = ({children}: {children: React.ReactNode}) => (
  <p style={{margin: '10px 0 0', fontSize: 13, lineHeight: 1.55, color: 'var(--text-secondary)'}}>{children}</p>
);

/** Lo que llega de `traer_de_la_v1`: lo traído, o por qué no se ha podido. */
export type ResultadoV1 = {paso: PasoV1} | {error: string};

export function DialogoVersionAnterior({resultado, cerrar}: {resultado: ResultadoV1 | null; cerrar: () => void}) {
  const [f, setF] = useState<Fase>({fase: 'aviso'});
  if (!resultado) return null;
  if ('error' in resultado) return <NoSeHaPodido error={resultado.error} cerrar={cerrar} />;
  const {paso} = resultado;
  const {traido} = paso;
  const ocupado = f.fase === 'quitando';

  const quitar = () => {
    setF({fase: 'quitando'});
    invoke('quitar_la_v1').then(
      () => setF({fase: 'quitada'}),
      (e) => setF({fase: 'error', mensaje: String(e)}),
    );
  };

  const nuevas = traido.clases + traido.notas;
  return (
    <Modal open onClose={ocupado ? undefined : cerrar} width={520} ariaLabel={t('v1.titulo')}>
      <div style={{padding: '20px 22px 16px', overflowY: 'auto', minHeight: 0}}>
        <h2 style={{margin: 0, fontSize: 15, fontWeight: 600}}>{t('v1.titulo')}</h2>
        <P>
          {nuevas > 0
            ? t('v1.traidas', {
                clases: t(traido.clases === 1 ? 'v1.clases_una' : 'v1.clases', {n: traido.clases}),
                notas: t(traido.notas === 1 ? 'v1.notas_una' : 'v1.notas', {n: traido.notas}),
              })
            : t('v1.ya_estaban')}
        </P>
        <div
          style={{
            marginTop: 8,
            fontFamily: 'var(--font-mono)',
            fontSize: 11.5,
            color: 'var(--text-tertiary)',
            overflowWrap: 'anywhere',
          }}>
          <div>{paso.plantillas}</div>
          <div>{paso.notas}</div>
        </div>
        {traido.renombrados.length > 0 && (
          <P>
            {t('v1.renombrados')} <span style={{color: 'var(--text-primary)'}}>{traido.renombrados.join(', ')}</span>
          </P>
        )}
        {paso.carpeta_cambiada && <P>{t('v1.carpeta_cambiada')}</P>}

        <div style={{marginTop: 14}}>
          {f.fase === 'aviso' && <P>{t(paso.pide_permiso ? 'v1.quitar_con_permiso' : 'v1.quitar')}</P>}
          {f.fase === 'quitando' && <Progreso rotulo={t('v1.quitando')} />}
          {f.fase === 'quitada' && <P>{t('v1.quitada')}</P>}
          {f.fase === 'error' && (
            <div role="alert" style={{fontSize: 13, lineHeight: 1.5}}>
              <div style={{fontWeight: 600, color: 'var(--danger)'}}>{t('v1.error')}</div>
              <div style={{color: 'var(--text-secondary)'}}>{f.mensaje}</div>
            </div>
          )}
        </div>
      </div>
      <div
        style={{
          display: 'flex',
          justifyContent: 'flex-end',
          gap: 8,
          padding: '12px 22px',
          borderTop: '1px solid var(--border)',
          background: 'var(--bg-sunk)',
        }}>
        {f.fase === 'quitada' ? (
          <Button onClick={cerrar}>{t('v1.entendido')}</Button>
        ) : (
          <>
            {!ocupado && (
              <Button variant="ghost" onClick={cerrar}>
                {t('v1.mas_tarde')}
              </Button>
            )}
            <Button onClick={quitar} disabled={ocupado}>
              {t(ocupado ? 'v1.quitando_boton' : f.fase === 'error' ? 'v1.reintentar' : 'v1.quitar_boton')}
            </Button>
          </>
        )}
      </div>
    </Modal>
  );
}

/** Copiar ha fallado: no se quita nada, y se vuelve a intentar al abrir. */
function NoSeHaPodido({error, cerrar}: {error: string; cerrar: () => void}) {
  return (
    <Modal open onClose={cerrar} width={520} ariaLabel={t('v1.titulo')}>
      <div role="alert" style={{padding: '20px 22px 16px'}}>
        <h2 style={{margin: 0, fontSize: 15, fontWeight: 600, color: 'var(--danger)'}}>{t('v1.no_copiadas')}</h2>
        <P>{error}</P>
        <P>{t('v1.no_copiadas_pie')}</P>
      </div>
      <div
        style={{
          display: 'flex',
          justifyContent: 'flex-end',
          padding: '12px 22px',
          borderTop: '1px solid var(--border)',
          background: 'var(--bg-sunk)',
        }}>
        <Button onClick={cerrar}>{t('v1.entendido')}</Button>
      </div>
    </Modal>
  );
}
