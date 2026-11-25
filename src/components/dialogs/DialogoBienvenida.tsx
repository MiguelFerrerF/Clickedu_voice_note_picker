/**
 * La bienvenida (H9): la primera vez que se abre la app, el recorrido en tres
 * pasos (traer, calificar, subir) y por dónde empezar.
 *
 * Por dónde, según lo que haya (decidido con el usuario el 2026-11-25): sin
 * plantillas, por Conexión, a traerlas de Clickedu; con plantillas (traídas,
 * o de la v1), por Clases, de donde se elige qué calificar. «Ahora no»,
 * siempre a mano. Cerrarla de cualquier forma la da por vista: se recupera
 * desde la Ayuda.
 */

import {Recorrido} from '@components/ayuda/Recorrido';
import {Modal} from '@components/dialogs/Modal';
import {Button} from '@components/ui/Button';
import {useClases} from '@modules/clases/useClases';
import {t} from '@modules/i18n';
import type {PantallaId} from '@/types/pantalla';

export interface DialogoBienvenidaProps {
  cerrar: () => void;
  /** Cierra y lleva a esa pantalla. */
  ir: (p: PantallaId) => void;
}

export function DialogoBienvenida({cerrar, ir}: DialogoBienvenidaProps) {
  const {lista} = useClases();
  const clases = lista?.clases.length;

  return (
    <Modal open onClose={cerrar} width={540} ariaLabel={t('bienvenida.titulo')}>
      <div style={{padding: '22px 24px 8px', overflowY: 'auto', minHeight: 0}}>
        <h2 style={{margin: 0, fontFamily: 'var(--font-display)', fontSize: 18, fontWeight: 600}}>
          {t('bienvenida.titulo')}
        </h2>
        <p style={{margin: '8px 0 0', fontSize: 13.5, lineHeight: 1.55, color: 'var(--text-secondary)'}}>
          {t('bienvenida.intro')}
        </p>
        <div style={{marginTop: 16}}>
          <Recorrido />
        </div>
        {clases !== undefined && (
          <p style={{margin: '18px 0 0', fontSize: 13.5, lineHeight: 1.55, color: 'var(--text-primary)'}}>
            {clases > 0
              ? clases === 1
                ? t('bienvenida.hay_una')
                : t('bienvenida.hay_clases', {n: clases})
              : t('bienvenida.no_hay_clases')}
          </p>
        )}
        <p style={{margin: '10px 0 0', fontSize: 12.5, color: 'var(--text-tertiary)'}}>{t('bienvenida.en_ayuda')}</p>
      </div>
      <div style={{display: 'flex', justifyContent: 'flex-end', gap: 8, padding: '14px 24px 18px'}}>
        <Button variant="ghost" onClick={cerrar}>
          {t('bienvenida.ahora_no')}
        </Button>
        {clases !== undefined &&
          (clases > 0 ? (
            <Button onClick={() => ir('clases')}>{t('bienvenida.ver_clases')}</Button>
          ) : (
            <Button onClick={() => ir('clickedu')}>{t('bienvenida.traer_clases')}</Button>
          ))}
      </div>
    </Modal>
  );
}
