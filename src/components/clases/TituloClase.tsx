/**
 * El título de Calificar y de Estadísticas: la clase y, si la trae de
 * Clickedu, su ítem (H6), cada uno en su desplegable («Clase · Ítem»). Las
 * dos pantallas comparten la clase y el ítem elegidos, que viven en `App`.
 *
 * Debajo, `IntroClase`: la evaluación y la ponderación del ítem, y el
 * fichero.
 */

import {Dropdown} from '@components/ui/Dropdown';
import {ponderacionTexto} from '@modules/clases/ponderacion';
import type {buscarItem, Clase, ItemElegido} from '@modules/clases/tipos';
import {t} from '@modules/i18n';

export interface TituloClaseProps {
  /** Las de la carpeta: las pendientes, al final. */
  clases: Clase[];
  actual: Clase;
  elegido: ReturnType<typeof buscarItem>;
  setClase: (fichero: string) => void;
  elegirItem: (fichero: string, item: ItemElegido | null) => void;
}

export function TituloClase({clases, actual, elegido, setClase, elegirItem}: TituloClaseProps) {
  const fichero = actual.fichero;
  return (
    // Cada desplegable, a su ancho y encogiendo solo si no cabe: una
    // rejilla y no flex, que el `Dropdown` ocupa el 100 % de su caja.
    <span
      style={{
        display: 'grid',
        gridTemplateColumns: actual.items
          ? 'minmax(0, max-content) max-content minmax(0, max-content)'
          : 'minmax(0, max-content)',
        justifyContent: 'start',
        alignItems: 'center',
        columnGap: 14,
      }}>
      <span style={{minWidth: 0}}>
        <Dropdown
          size="titulo"
          value={fichero}
          onChange={setClase}
          title={t('calificar.elegir_clase')}
          options={clases.map((c) => ({
            value: c.fichero,
            label: c.nombre,
            grupo: c.pendiente ? t('clases.pendientes') : undefined,
          }))}
        />
      </span>
      {actual.items && (
        <>
          <span aria-hidden style={{flexShrink: 0, color: 'var(--text-tertiary)'}}>
            ·
          </span>
          <span style={{minWidth: 0}}>
            <Dropdown
              size="titulo"
              value={elegido ? `${elegido.evaluacion.id}|${elegido.item.id}` : ''}
              onChange={(v) => {
                const [evaluacion, item] = v.split('|');
                elegirItem(fichero, item ? {evaluacion, item} : null);
              }}
              title={t('calificar.elegir_item')}
              options={[
                {value: '', label: t('calificar.sin_item')},
                ...actual.items.evaluaciones.flatMap((e) =>
                  e.items.map((i) => ({value: `${e.id}|${i.id}`, label: i.nombre, grupo: e.nombre})),
                ),
              ]}
            />
          </span>
        </>
      )}
    </span>
  );
}

export function IntroClase({actual, elegido}: Pick<TituloClaseProps, 'actual' | 'elegido'>) {
  return (
    <span style={{display: 'flex', flexWrap: 'wrap', alignItems: 'baseline', columnGap: 10}}>
      {actual.items && (
        <span>
          {elegido
            ? [elegido.evaluacion.nombre, ponderacionTexto(elegido.item)].filter(Boolean).join(' · ')
            : t('calificar.sin_item_ayuda')}
        </span>
      )}
      <span style={{fontFamily: 'var(--font-mono)', fontSize: 12}}>{actual.fichero}</span>
    </span>
  );
}
