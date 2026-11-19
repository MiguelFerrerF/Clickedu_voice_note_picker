/**
 * El registro del dictado, en Ajustes (D8): cuánto hay guardado, preparar
 * el informe para mandarlo y borrarlo.
 *
 * Lo guardado no sale del ordenador. El informe sí puede salir, porque no
 * lleva nombres de alumnos ni voz; se deja en `Documentos\Gestor de
 * Notas\informes` y el profesor lo manda como quiera, después de abrirlo y
 * ver lo que lleva. Preparado, se dice en verde, que se apaga solo (D6).
 */

import {invoke} from '@tauri-apps/api/core';
import {useEffect, useState} from 'react';

import {Modal} from '@components/dialogs/Modal';
import {Button} from '@components/ui/Button';
import {Card} from '@components/ui/Card';
import {Icon, ICONOS} from '@components/ui/Icon';
import {t} from '@modules/i18n';
import type {ResumenRegistro} from '@modules/registro/tipos';

const APAGAR_MS = 6000;

/** `visible`: la pantalla se está viendo. Al volver a ella, se cuenta otra vez. */
export function TarjetaRegistro({visible}: {visible: boolean}) {
  const [resumen, setResumen] = useState<ResumenRegistro>();
  const [informe, setInforme] = useState<string>();
  const [encendido, setEncendido] = useState(false);
  const [preparando, setPreparando] = useState(false);
  const [borrar, setBorrar] = useState(false);
  const [fallo, setFallo] = useState<string>();

  const contar = () => invoke<ResumenRegistro>('resumen_registro').then(setResumen, (e) => setFallo(String(e)));
  useEffect(() => {
    if (visible) contar();
  }, [visible]);
  useEffect(() => {
    if (!encendido) return;
    const apagar = setTimeout(() => setEncendido(false), APAGAR_MS);
    return () => clearTimeout(apagar);
  }, [encendido, informe]);

  function preparar() {
    setFallo(undefined);
    setPreparando(true);
    invoke<string>('preparar_informe')
      .then(
        (ruta) => {
          setInforme(ruta);
          setEncendido(true);
        },
        (e) => setFallo(String(e)),
      )
      .finally(() => setPreparando(false));
  }

  function borrarTodo() {
    setBorrar(false);
    setFallo(undefined);
    invoke('borrar_registro').then(
      () => {
        setInforme(undefined);
        contar();
      },
      (e) => setFallo(String(e)),
    );
  }

  const hay = resumen && (resumen.dictados > 0 || resumen.fallos > 0);
  const cifras: [string, number][] = resumen
    ? [
        [t('registro.dictados'), resumen.dictados],
        [t('registro.seguros'), resumen.seguros],
        [t('registro.preguntados'), resumen.preguntados],
        [t('registro.no_entendidos'), resumen.noEntendidos],
        [t('registro.corregidos'), resumen.corregidos],
        [t('registro.fallos'), resumen.fallos],
      ]
    : [];

  return (
    <Card title={t('registro.titulo')} subtitle={t('registro.sub')} style={{marginTop: 16}}>
      {resumen && !hay && <p style={{margin: 0, fontSize: 13, color: 'var(--text-tertiary)'}}>{t('registro.vacio')}</p>}
      {hay && (
        <>
          <div
            style={{display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(130px, 1fr))', gap: '10px 16px'}}>
            {cifras.map(([rotulo, n]) => (
              <div key={rotulo} style={{minWidth: 0}}>
                <div style={{fontFamily: 'var(--font-mono)', fontSize: 18, color: 'var(--text-primary)'}}>{n}</div>
                <div style={{fontSize: 11.5, color: 'var(--text-tertiary)'}}>{rotulo}</div>
              </div>
            ))}
          </div>
          {resumen.desde !== null && (
            <p style={{margin: 0, fontSize: 12, color: 'var(--text-tertiary)'}}>
              {t('registro.desde', {fecha: new Date(resumen.desde * 1000).toLocaleDateString('es-ES')})}
            </p>
          )}
          <div style={{display: 'flex', flexWrap: 'wrap', gap: 8}}>
            <Button variant="ghost" onClick={preparar} disabled={preparando}>
              <Icon d={ICONOS.exportar} size={14} />
              {t('registro.preparar')}
            </Button>
            <Button variant="ghost" onClick={() => setBorrar(true)}>
              {t('registro.borrar')}
            </Button>
          </div>
        </>
      )}
      {informe && (
        <div
          role="status"
          style={{
            display: 'flex',
            alignItems: 'center',
            flexWrap: 'wrap',
            gap: '6px 12px',
            fontSize: 13,
            color: encendido ? 'var(--success)' : 'var(--text-secondary)',
            transition: 'color 600ms',
          }}>
          <span style={{display: 'inline-flex', alignItems: 'center', gap: 6, minWidth: 0}}>
            <Icon d={ICONOS.comprobado} size={14} style={{flexShrink: 0}} />
            <span style={{overflowWrap: 'anywhere'}}>
              {t('registro.preparado', {fichero: informe.split(/[\\/]/).pop() ?? informe})}
            </span>
          </span>
          <Button
            variant="ghost"
            onClick={() => invoke('mostrar_informe', {ruta: informe}).catch((e) => setFallo(String(e)))}>
            <Icon d={ICONOS.carpeta} size={14} />
            {t('registro.mostrar')}
          </Button>
        </div>
      )}
      {fallo && (
        <p role="alert" style={{margin: 0, fontSize: 13, color: 'var(--danger)'}}>
          {fallo}
        </p>
      )}
      <Modal open={borrar} onClose={() => setBorrar(false)} width={460} ariaLabel={t('registro.borrar_titulo')}>
        <div style={{padding: '20px 22px'}}>
          <h2 style={{margin: 0, fontSize: 15, fontWeight: 600}}>{t('registro.borrar_titulo')}</h2>
          <p style={{margin: '10px 0 0', fontSize: 13, lineHeight: 1.55, color: 'var(--text-secondary)'}}>
            {t('registro.borrar_texto')}
          </p>
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
          <Button variant="ghost" onClick={() => setBorrar(false)}>
            {t('registro.cancelar')}
          </Button>
          <Button variant="stop" onClick={borrarTodo}>
            {t('registro.borrar_si')}
          </Button>
        </div>
      </Modal>
    </Card>
  );
}
