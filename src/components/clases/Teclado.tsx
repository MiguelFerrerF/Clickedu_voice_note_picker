/**
 * Las teclas de Calificar, una por fila (HUI): se abren con el ⓘ de encima
 * de la tabla, en vez de una línea de ayuda siempre a la vista, que con todo
 * lo demás saturaba la pantalla.
 *
 * Van las de la tabla y las del dictado, para tenerlas en un sitio. Es un
 * panel neutro, sin color: no es un evento, es una consulta. La ayuda de
 * cada pantalla, con sus tours, es de H9.
 */

import {Icon, ICONOS} from '@components/ui/Icon';
import {conTeclas} from '@components/ui/Tecla';
import {t} from '@modules/i18n';

/** Los rótulos de las teclas, para `conTeclas`. */
const teclas = () => ({
  enter: t('cuadricula.tecla_enter'),
  abajo: t('cuadricula.tecla_abajo'),
  arriba: t('cuadricula.tecla_arriba'),
  supr: t('cuadricula.tecla_supr'),
  esc: t('dictado.tecla_esc'),
  espacio: t('dictado.tecla_espacio'),
  uno: '1',
  cuatro: '4',
});

/** Cada grupo: su título y sus filas, cada una con sus teclas y lo que hacen. */
const GRUPOS = [
  {
    titulo: 'teclado.en_la_tabla',
    filas: ['teclado.bajar', 'teclado.subir', 'teclado.borrar', 'teclado.devolver'],
  },
  {
    titulo: 'teclado.al_dictar',
    filas: ['teclado.dictar', 'teclado.elegir', 'teclado.descartar'],
  },
] as const;

export const ID_TECLADO = 'teclado-calificar';

export function Teclado({cerrar}: {cerrar: () => void}) {
  const rotulos = teclas();
  return (
    <section
      id={ID_TECLADO}
      aria-label={t('teclado.titulo')}
      style={{
        position: 'relative',
        display: 'flex',
        flexWrap: 'wrap',
        gap: '12px 40px',
        marginBottom: 12,
        padding: '12px 44px 12px 16px',
        border: '1px solid var(--border)',
        borderRadius: 8,
        background: 'var(--bg-card)',
      }}>
      {GRUPOS.map((g) => (
        <div key={g.titulo} style={{minWidth: 0}}>
          <h2
            style={{
              margin: '0 0 6px',
              fontSize: 10.5,
              fontWeight: 600,
              letterSpacing: 0.8,
              textTransform: 'uppercase',
              color: 'var(--text-tertiary)',
            }}>
            {t(g.titulo)}
          </h2>
          <dl style={{margin: 0, display: 'grid', gridTemplateColumns: 'auto minmax(0, 1fr)', gap: '6px 14px'}}>
            {g.filas.map((f) => (
              <div key={f} style={{display: 'contents'}}>
                <dt style={{whiteSpace: 'nowrap', fontSize: 12, color: 'var(--text-secondary)'}}>
                  {conTeclas(t(`${f}_teclas`), rotulos)}
                </dt>
                <dd style={{margin: 0, fontSize: 13, color: 'var(--text-primary)', lineHeight: '19px'}}>{t(f)}</dd>
              </div>
            ))}
          </dl>
        </div>
      ))}
      <p style={{flexBasis: '100%', margin: 0, fontSize: 12, color: 'var(--text-secondary)'}}>{t('teclado.nota')}</p>
      <button
        type="button"
        onClick={cerrar}
        aria-label={t('teclado.cerrar')}
        title={t('teclado.cerrar')}
        style={{
          position: 'absolute',
          top: 8,
          right: 8,
          display: 'inline-flex',
          padding: 4,
          border: 'none',
          borderRadius: 6,
          background: 'none',
          color: 'var(--text-tertiary)',
          cursor: 'pointer',
        }}>
        <Icon d={ICONOS.cerrar} size={14} />
      </button>
    </section>
  );
}
