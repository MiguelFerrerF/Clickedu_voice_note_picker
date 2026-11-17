/**
 * Barra lateral — el logo y el nombre, las secciones de navegación, y la
 * versión abajo.
 *
 * Es la de library-app con lo suyo quitado. Lo que se añade es el **logo**:
 * allí no iba porque el de la Quality Test App es la marca de inBiot, pero
 * aquí el logo es el del propio producto y es de donde sale el color.
 *
 * Tres secciones:
 *
 * - **Aula**: lo de todos los días. Calificar, Clases y Estadísticas.
 * - **Clickedu**: la conexión, traer plantillas y subir notas.
 * - **Aplicación**: Ajustes y, solo en desarrollo, el Muestrario.
 *
 * Al pie, la versión. «Buscar actualizaciones» llega en H7.
 */

import logo from '@/assets/logo.png';
import {NavItem} from './NavItem';
import {SidebarSection} from './SidebarSection';
import {ICONOS} from '@components/ui/Icon';
import {VERSION_ACTUAL} from '@modules/app/version';
import {t} from '@modules/i18n';
import {useIdioma} from '@modules/i18n/useIdioma';
import type {PantallaId} from '@/types/pantalla';

export interface SidebarProps {
  pantalla: PantallaId;
  setPantalla: (p: PantallaId) => void;
}

export function Sidebar({pantalla, setPantalla}: SidebarProps) {
  useIdioma();

  const entrada = (id: PantallaId, icono: string | readonly string[], rotulo: string) => (
    <NavItem
      activo={pantalla === id}
      icono={icono}
      rotulo={rotulo}
      onClick={() => setPantalla(id)}
    />
  );

  return (
    <aside
      style={{
        width: 'var(--nav-w)',
        flexShrink: 0,
        background: 'var(--bg-sidebar)',
        borderRight: '1px solid var(--border)',
        display: 'flex',
        flexDirection: 'column',
        padding: '14px 16px',
        overflow: 'hidden',
      }}>
      <div
        style={{
          display: 'flex',
          alignItems: 'center',
          gap: 10,
          paddingBottom: 16,
          marginBottom: 4,
          borderBottom: '1px solid var(--border)',
          flexShrink: 0,
        }}>
        <img src={logo} alt="" width={30} height={30} style={{flexShrink: 0, objectFit: 'contain'}} />
        <div style={{minWidth: 0}}>
          <div
            style={{
              fontFamily: 'var(--font-display)',
              fontSize: 15,
              fontWeight: 600,
              color: 'var(--text-primary)',
              letterSpacing: '-0.01em',
            }}>
            {t('app.nombre')}
          </div>
          <div style={{fontSize: 11, color: 'var(--text-tertiary)', marginTop: 2}}>
            {t('app.subtitulo')}
          </div>
        </div>
      </div>

      <div
        style={{
          flex: 1,
          overflowY: 'auto',
          overflowX: 'hidden',
          minHeight: 0,
          margin: '0 -2px',
          padding: '0 2px',
        }}>
        <SidebarSection id="aula" titulo={t('nav.aula')}>
          {entrada('calificar', ICONOS.calificar, t('nav.calificar'))}
          {entrada('clases', ICONOS.clases, t('nav.clases'))}
          {entrada('estadisticas', ICONOS.estadisticas, t('nav.estadisticas'))}
        </SidebarSection>

        <SidebarSection id="clickedu" titulo={t('nav.clickedu_seccion')}>
          {entrada('clickedu', ICONOS.clickedu, t('nav.clickedu'))}
        </SidebarSection>

        <SidebarSection id="aplicacion" titulo={t('nav.aplicacion')}>
          {entrada('ajustes', ICONOS.ajustes, t('nav.ajustes'))}
          {import.meta.env.DEV && entrada('muestrario', ICONOS.muestrario, t('nav.muestrario'))}
        </SidebarSection>
      </div>

      <div
        style={{
          flexShrink: 0,
          paddingTop: 12,
          marginTop: 8,
          borderTop: '1px solid var(--border)',
          fontFamily: 'var(--font-mono)',
          fontSize: 10.5,
          color: 'var(--text-tertiary)',
        }}>
        {t('comun.version', {version: VERSION_ACTUAL})}
      </div>
    </aside>
  );
}
