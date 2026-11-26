/**
 * «Clickedu, al día», en Ajustes (H11): si lo de Clickedu se refresca solo
 * y cada cuánto, y cómo fue el último.
 *
 * El interruptor y los minutos van en **un** desplegable («Cada 15
 * minutos», «Cada 30 minutos», «Cada hora», «No: solo al traer»): sin un
 * componente nuevo, como pide la regla de traerlos de library-app, que no
 * tiene interruptor. Apagado, se recuerdan los minutos de antes.
 */

import {invoke} from '@tauri-apps/api/core';
import {useState} from 'react';

import {LineaRefresco} from '@components/clickedu/Refresco';
import {Card} from '@components/ui/Card';
import {Dropdown} from '@components/ui/Dropdown';
import {MINUTOS, useEstadoRefresco, type EstadoRefresco} from '@modules/clickedu/refresco';
import {t} from '@modules/i18n';

const NUNCA = 'nunca';

export function TarjetaRefresco() {
  const [estado, setEstado] = useEstadoRefresco();
  const [fallo, setFallo] = useState<string>();

  function elegir(valor: string) {
    if (!estado) return;
    setFallo(undefined);
    const encendido = valor !== NUNCA;
    const minutos = encendido ? Number(valor) : estado.minutos;
    invoke<EstadoRefresco>('poner_refresco', {encendido, minutos}).then(setEstado, (e) => setFallo(String(e)));
  }

  const opciones = [
    ...MINUTOS.map((n) => ({
      value: String(n),
      label: n === 60 ? t('refresco.cada_hora') : t('refresco.cada_minutos', {n}),
    })),
    {value: NUNCA, label: t('refresco.nunca')},
  ];

  return (
    <Card title={t('refresco.titulo')} subtitle={t('refresco.sub')}>
      {estado && (
        <div style={{maxWidth: 480}}>
          <Dropdown
            value={estado.encendido ? String(estado.minutos) : NUNCA}
            onChange={elegir}
            title={t('refresco.cada')}
            options={opciones}
          />
        </div>
      )}
      <LineaRefresco estado={estado} />
      {fallo && (
        <p role="alert" style={{margin: 0, fontSize: 13, color: 'var(--danger)'}}>
          {fallo}
        </p>
      )}
    </Card>
  );
}
