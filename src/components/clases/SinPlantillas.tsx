/**
 * Lo que se ve cuando no hay ninguna clase: la carpeta está vacía, o la
 * elegida ya no existe. Dice dónde se busca y deja abrirla.
 */

import {invoke} from '@tauri-apps/api/core';

import {Button} from '@components/ui/Button';
import {Icon, ICONOS} from '@components/ui/Icon';
import type {CarpetaPlantillas} from '@modules/clases/tipos';
import {t} from '@modules/i18n';

export function SinPlantillas({carpeta, irAAjustes}: {carpeta: CarpetaPlantillas; irAAjustes: () => void}) {
  return (
    <div
      style={{
        display: 'flex',
        gap: 16,
        alignItems: 'flex-start',
        padding: '20px 22px',
        background: 'var(--bg-card)',
        border: '1px solid var(--border)',
        borderRadius: 10,
        maxWidth: 820,
      }}>
      <span style={{color: carpeta.existe ? 'var(--accent-ink)' : 'var(--danger)', flexShrink: 0, marginTop: 1}}>
        <Icon d={ICONOS.carpeta} size={22} />
      </span>
      <div style={{minWidth: 0}}>
        <p style={{margin: 0, fontSize: 13.5, fontWeight: 600, color: 'var(--text-primary)'}}>
          {carpeta.existe ? t('clases.sin_plantillas') : t('clases.carpeta_no_existe')}
        </p>
        <p style={{margin: '6px 0 0', fontSize: 13, lineHeight: 1.55, color: 'var(--text-secondary)'}}>
          {carpeta.existe ? t('clases.sin_plantillas_ayuda') : t('clases.carpeta_no_existe_ayuda')}
        </p>
        <div style={{marginTop: 10}}>
          <Ruta ruta={carpeta.ruta} />
        </div>
        <div style={{display: 'flex', flexWrap: 'wrap', gap: 8, marginTop: 14}}>
          {carpeta.existe && (
            <Button onClick={() => invoke('abrir_carpeta_plantillas').catch(() => {})}>
              <Icon d={ICONOS.carpeta} size={14} />
              {t('clases.abrir_carpeta')}
            </Button>
          )}
          <Button variant="ghost" onClick={irAAjustes}>
            {t('clases.cambiar_carpeta')}
          </Button>
        </div>
      </div>
    </div>
  );
}

/** Una ruta de carpeta, en mono y cortada con «…» si no cabe. */
export function Ruta({ruta}: {ruta: string}) {
  return (
    <div
      title={ruta}
      style={{
        fontFamily: 'var(--font-mono)',
        fontSize: 12,
        color: 'var(--text-secondary)',
        overflow: 'hidden',
        textOverflow: 'ellipsis',
        whiteSpace: 'nowrap',
      }}>
      {ruta}
    </div>
  );
}
