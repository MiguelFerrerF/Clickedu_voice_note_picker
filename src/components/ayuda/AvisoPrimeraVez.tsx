/**
 * El aviso de la primera vez en una pantalla (H9): dos o tres líneas de qué
 * se hace aquí, «Más en la Ayuda» y la ✕.
 *
 * Lo pone `Cabecera`, debajo del título, en las pantallas de `CON_AVISO`.
 * Se da por visto **al cerrarlo**, no al verlo: si se pasa de largo, vuelve
 * la próxima vez. En el acento, porque es un evento que se apaga (D6), no un
 * estado de la pantalla.
 */

import {Icon, ICONOS} from '@components/ui/Icon';
import {tieneAviso, useAyuda, usePantalla} from '@modules/ayuda';
import {t} from '@modules/i18n';

export function AvisoPrimeraVez() {
  const pantalla = usePantalla();
  const {vista, marcar, abrir} = useAyuda();
  if (!pantalla || !tieneAviso(pantalla) || !vista || vista.has(pantalla)) return null;

  return (
    <section
      aria-label={t('ayuda.aviso_titulo')}
      style={{
        position: 'relative',
        display: 'flex',
        gap: 12,
        alignItems: 'flex-start',
        marginBottom: 20,
        padding: '12px 44px 12px 14px',
        borderRadius: 8,
        background: 'var(--accent-wash)',
      }}>
      <span style={{color: 'var(--accent-ink)', flexShrink: 0, marginTop: 1}}>
        <Icon d={ICONOS.info} size={16} />
      </span>
      <div style={{minWidth: 0, maxWidth: 760}}>
        <p style={{margin: 0, fontSize: 13, lineHeight: 1.55, color: 'var(--text-primary)'}}>
          {t(`ayuda.aviso_${pantalla}`)}
        </p>
        <button
          type="button"
          onClick={abrir}
          style={{
            all: 'unset',
            marginTop: 4,
            fontSize: 12.5,
            color: 'var(--accent-ink)',
            textDecoration: 'underline',
            textUnderlineOffset: 2,
            cursor: 'pointer',
          }}>
          {t('ayuda.mas_en_la_ayuda')}
        </button>
      </div>
      <button
        type="button"
        onClick={() => marcar(pantalla)}
        aria-label={t('ayuda.entendido')}
        title={t('ayuda.entendido')}
        style={{
          position: 'absolute',
          top: 8,
          right: 8,
          display: 'inline-flex',
          padding: 4,
          border: 'none',
          borderRadius: 6,
          background: 'none',
          color: 'var(--accent-ink)',
          cursor: 'pointer',
        }}>
        <Icon d={ICONOS.cerrar} size={14} />
      </button>
    </section>
  );
}
