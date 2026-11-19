/**
 * El micrófono por el que se dicta, en Ajustes (HUI). Si no se elige, el de
 * Windows; si se conectan unos auriculares con micrófono, se eligen aquí.
 *
 * La lista se mira otra vez cada pocos segundos mientras Ajustes está a la
 * vista: al enchufar unos auriculares, aparecen sin hacer nada. Si el
 * elegido no está conectado, se sigue enseñando, se dice, y mientras se
 * dicta por el de Windows (no se cambia en silencio).
 */

import {invoke} from '@tauri-apps/api/core';
import {useEffect, useState} from 'react';

import {Punto} from '@components/clases/Avisos';
import {Card} from '@components/ui/Card';
import {Dropdown} from '@components/ui/Dropdown';
import {t} from '@modules/i18n';

/** El espejo de `commands::voz::Microfonos`. */
interface Microfonos {
  disponibles: Elegido[];
  deWindows: string | null;
  elegido: Elegido | null;
}

/** El espejo de `ajustes::MicrofonoElegido`. */
interface Elegido {
  id: string;
  nombre: string;
}

/** Cada cuánto se mira si se ha conectado o quitado alguno. */
const MIRAR_MS = 3000;

/** El valor de «el de Windows» en el desplegable. */
const DE_WINDOWS = '';

/** `visible`: Ajustes se está viendo; solo entonces se mira la lista. */
export function TarjetaMicrofono({visible}: {visible: boolean}) {
  const [microfonos, setMicrofonos] = useState<Microfonos>();
  const [fallo, setFallo] = useState<string>();

  const mirar = () =>
    invoke<Microfonos>('listar_microfonos').then(
      (m) => {
        setMicrofonos(m);
        setFallo(undefined);
      },
      (e) => setFallo(String(e)),
    );
  useEffect(() => {
    if (!visible) return;
    mirar();
    const cada = setInterval(mirar, MIRAR_MS);
    return () => clearInterval(cada);
  }, [visible]);

  function elegir(id: string) {
    if (!microfonos) return;
    const elegido = id === DE_WINDOWS ? null : (microfonos.disponibles.find((d) => d.id === id) ?? null);
    setMicrofonos({...microfonos, elegido});
    invoke('elegir_microfono', {microfono: elegido}).then(mirar, (e) => setFallo(String(e)));
  }

  const deWindows = microfonos?.disponibles.find((d) => d.id === microfonos.deWindows);
  const elegido = microfonos?.elegido;
  const falta = elegido && !microfonos.disponibles.some((d) => d.id === elegido.id) ? elegido : undefined;
  const opciones = [
    {
      value: DE_WINDOWS,
      label: deWindows ? t('microfono.de_windows_ahora', {nombre: deWindows.nombre}) : t('microfono.de_windows'),
    },
    ...(microfonos?.disponibles ?? []).map((d) => ({value: d.id, label: d.nombre})),
    ...(falta ? [{value: falta.id, label: t('microfono.no_conectado', {nombre: falta.nombre})}] : []),
  ];

  return (
    <Card title={t('microfono.titulo')} subtitle={t('microfono.sub')}>
      {microfonos && (
        <div style={{maxWidth: 480}}>
          <Dropdown
            value={elegido?.id ?? DE_WINDOWS}
            onChange={elegir}
            title={t('microfono.elegir')}
            options={opciones}
          />
        </div>
      )}
      {falta && (
        <p style={{margin: 0, display: 'flex', alignItems: 'baseline', gap: 8, fontSize: 13}}>
          <Punto />
          <span style={{overflowWrap: 'anywhere'}}>{t('microfono.falta', {nombre: falta.nombre})}</span>
        </p>
      )}
      {microfonos && microfonos.disponibles.length === 0 && (
        <p style={{margin: 0, display: 'flex', alignItems: 'baseline', gap: 8, fontSize: 13}}>
          <Punto />
          {t('microfono.ninguno')}
        </p>
      )}
      {fallo && (
        <p role="alert" style={{margin: 0, fontSize: 13, color: 'var(--danger)'}}>
          {t('microfono.no_se_ven', {error: fallo})}
        </p>
      )}
    </Card>
  );
}
