/**
 * Ayuda (H9): cómo funciona cada cosa de la app, en un sitio. Pedida por el
 * usuario el 2026-11-19; es el «?» de library-app.
 *
 * - **El recorrido**: traer, calificar y subir, como en la bienvenida, que
 *   se puede volver a ver desde aquí.
 * - **Cada pantalla**: qué se hace en ella, con un botón para ir.
 * - **El dictado** y **las teclas** (las mismas del ⓘ de Calificar).
 * - **Lo que no sale del ordenador** (D2, D3, D8) y **si algo no va**.
 *
 * Es una consulta, no un evento: neutra, sin acento más que en los botones.
 * Las tarjetas pequeñas, en dos columnas cuando caben; los párrafos, con
 * tope de ancho.
 */

import {Recorrido} from '@components/ayuda/Recorrido';
import {ListaTeclas} from '@components/clases/Teclado';
import {Cabecera} from '@components/layout/Cabecera';
import {Button} from '@components/ui/Button';
import {Card} from '@components/ui/Card';
import {Icon, ICONOS} from '@components/ui/Icon';
import {t, type TranslationKey} from '@modules/i18n';
import {useIdioma} from '@modules/i18n/useIdioma';
import type {PantallaId} from '@/types/pantalla';

/** Las pantallas, en el orden de la barra lateral. */
const PANTALLAS: readonly {id: PantallaId; icono: string | readonly string[]; nombre: TranslationKey}[] = [
  {id: 'calificar', icono: ICONOS.calificar, nombre: 'nav.calificar'},
  {id: 'clases', icono: ICONOS.clases, nombre: 'nav.clases'},
  {id: 'estadisticas', icono: ICONOS.estadisticas, nombre: 'nav.estadisticas'},
  {id: 'clickedu', icono: ICONOS.clickedu, nombre: 'nav.clickedu'},
  {id: 'ajustes', icono: ICONOS.ajustes, nombre: 'nav.ajustes'},
];

const PARRAFO: React.CSSProperties = {
  margin: 0,
  maxWidth: 760,
  fontSize: 13,
  lineHeight: 1.55,
  color: 'var(--text-secondary)',
};

/** Una lista de párrafos cortos, con su punto. */
function Lista({claves}: {claves: readonly TranslationKey[]}) {
  return (
    <ul style={{...PARRAFO, paddingLeft: 18, display: 'grid', gap: 6}}>
      {claves.map((c) => (
        <li key={c}>{t(c)}</li>
      ))}
    </ul>
  );
}

export interface AyudaProps {
  ir: (p: PantallaId) => void;
  verBienvenida: () => void;
}

export function Ayuda({ir, verBienvenida}: AyudaProps) {
  useIdioma();

  return (
    <div style={{padding: '28px 32px'}}>
      <Cabecera
        titulo={t('ayuda.titulo')}
        intro={t('ayuda.intro')}
        derecha={
          <Button variant="ghost" onClick={verBienvenida}>
            {t('ayuda.ver_bienvenida')}
          </Button>
        }
      />
      <div style={{display: 'flex', flexDirection: 'column', gap: 16}}>
        <Card title={t('ayuda.recorrido')}>
          <Recorrido enFila />
        </Card>

        <Card title={t('ayuda.pantallas')}>
          <div style={{display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(260px, 1fr))', gap: 12}}>
            {PANTALLAS.map((p) => (
              <section
                key={p.id}
                aria-label={t(p.nombre)}
                style={{
                  display: 'flex',
                  flexDirection: 'column',
                  gap: 8,
                  padding: '12px 14px',
                  border: '1px solid var(--border)',
                  borderRadius: 8,
                  minWidth: 0,
                }}>
                <h4
                  style={{
                    margin: 0,
                    display: 'flex',
                    alignItems: 'center',
                    gap: 8,
                    fontSize: 13.5,
                    fontWeight: 600,
                    color: 'var(--text-primary)',
                  }}>
                  <span style={{color: 'var(--text-secondary)', display: 'inline-flex'}}>
                    <Icon d={p.icono} size={16} />
                  </span>
                  {t(p.nombre)}
                </h4>
                <p style={{...PARRAFO, flex: 1}}>{t(`ayuda.pantalla_${p.id}` as TranslationKey)}</p>
                <div>
                  <Button variant="ghost" onClick={() => ir(p.id)}>
                    {t('ayuda.ir_a', {pantalla: t(p.nombre)})}
                  </Button>
                </div>
              </section>
            ))}
          </div>
        </Card>

        <div style={{display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(min(420px, 100%), 1fr))', gap: 16}}>
          <Card title={t('ayuda.dictado')}>
            <Lista
              claves={['ayuda.dictado_como', 'ayuda.dictado_dudas', 'ayuda.dictado_modelo', 'ayuda.dictado_microfono']}
            />
          </Card>
          <Card title={t('teclado.titulo')}>
            <div style={{display: 'flex', flexWrap: 'wrap', gap: '12px 40px'}}>
              <ListaTeclas />
            </div>
          </Card>
          <Card title={t('ayuda.privado')}>
            <Lista
              claves={[
                'ayuda.privado_voz',
                'ayuda.privado_cuenta',
                'ayuda.privado_plantillas',
                'ayuda.privado_registro',
              ]}
            />
          </Card>
          <Card title={t('ayuda.si_falla')}>
            <Lista
              claves={['ayuda.falla_subir', 'ayuda.falla_dictado', 'ayuda.falla_plantilla', 'ayuda.falla_version']}
            />
          </Card>
        </div>
      </div>
    </div>
  );
}
