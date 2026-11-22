/**
 * «Tu cuenta», arriba en la pantalla Conexión (H4, diseño del 2026-11-20 en
 * `docs/01`).
 *
 * - **Sin cuenta**: usuario, contraseña y el archivo de seguridad.
 *   «Entrar» entra de verdad y **solo se guarda si ha entrado**.
 * - **Con cuenta**: el usuario y el archivo, en gris (estar conectado es lo
 *   normal, D6), y tres botones fantasma: cambiar el archivo (elegirlo ya
 *   vuelve a entrar con él), cambiar la contraseña y olvidar la cuenta.
 * - «Has entrado en Clickedu», en verde, **se apaga solo**: es un evento.
 * - Los fallos, en rojo y con qué hacer.
 *
 * **La contraseña guardada no llega nunca aquí**: para cambiarla se escribe
 * otra. El ojo enseña solo lo que se está escribiendo.
 */

import {invoke} from '@tauri-apps/api/core';
import {useEffect, useState} from 'react';

import {Modal} from '@components/dialogs/Modal';
import {Button} from '@components/ui/Button';
import {Card} from '@components/ui/Card';
import {Icon, ICONOS} from '@components/ui/Icon';
import {TextInput} from '@components/ui/TextInput';
import type {CuentaClickedu, FalloClickedu} from '@modules/clickedu/tipos';
import {t} from '@modules/i18n';

const APAGAR_MS = 6000;
/** Los campos son controles: no crecen con la ventana. */
const ANCHO_CAMPOS = 420;

type Modo = 'nueva' | 'contrasena';

/** `alCambiar`: si hay cuenta guardada, para lo de debajo (H5). */
export function TarjetaCuenta({alCambiar}: {alCambiar?: (hay: boolean) => void}) {
  /** `undefined`: aún no se sabe; `null`: no hay. */
  const [cuenta, setCuenta] = useState<CuentaClickedu | null>();
  const [modo, setModo] = useState<Modo>();
  const [usuario, setUsuario] = useState('');
  const [contrasena, setContrasena] = useState('');
  const [archivo, setArchivo] = useState<string>();
  const [entrando, setEntrando] = useState(false);
  const [entrado, setEntrado] = useState(false);
  const [encendido, setEncendido] = useState(false);
  const [olvidar, setOlvidar] = useState(false);
  const [fallo, setFallo] = useState<FalloClickedu | string>();

  useEffect(() => {
    invoke<CuentaClickedu | null>('cuenta_clickedu').then(setCuenta, (e) => {
      setCuenta(null);
      setFallo(String(e));
    });
  }, []);
  useEffect(() => {
    if (cuenta !== undefined) alCambiar?.(cuenta !== null);
  }, [cuenta, alCambiar]);
  useEffect(() => {
    if (!encendido) return;
    const apagar = setTimeout(() => setEncendido(false), APAGAR_MS);
    return () => clearTimeout(apagar);
  }, [encendido]);

  /** Lo que no se pasa, de la cuenta guardada. */
  async function entrar(con: {usuario?: string; contrasena?: string; archivo?: string}) {
    setFallo(undefined);
    setEntrado(false);
    setEntrando(true);
    try {
      const v = await invoke<CuentaClickedu>('entrar_clickedu', {
        usuario: con.usuario ?? null,
        contrasena: con.contrasena ?? null,
        archivo: con.archivo ?? null,
      });
      setCuenta(v);
      setModo(undefined);
      setContrasena('');
      setArchivo(undefined);
      setEntrado(true);
      setEncendido(true);
    } catch (e) {
      setFallo(e as FalloClickedu | string);
    } finally {
      setEntrando(false);
    }
  }

  async function elegir(): Promise<string | undefined> {
    setFallo(undefined);
    try {
      return (await invoke<string | null>('elegir_archivo_seguridad')) ?? undefined;
    } catch (e) {
      setFallo(String(e));
    }
  }

  function olvidarla() {
    setOlvidar(false);
    setFallo(undefined);
    invoke('olvidar_cuenta_clickedu').then(
      () => {
        setCuenta(null);
        setEntrado(false);
        setUsuario('');
        setContrasena('');
        setArchivo(undefined);
      },
      (e) => setFallo(String(e)),
    );
  }

  const formulario = cuenta === null ? 'nueva' : modo;
  const listo =
    formulario === 'nueva' ? usuario.trim() !== '' && contrasena !== '' && archivo !== undefined : contrasena !== '';

  return (
    <Card title={t('clickedu.cuenta')} subtitle={cuenta === null ? t('clickedu.cuenta_sub') : undefined}>
      {cuenta && !formulario && (
        <Guardada
          cuenta={cuenta}
          entrando={entrando}
          cambiarArchivo={async () => {
            const ruta = await elegir();
            if (ruta) entrar({archivo: ruta});
          }}
          cambiarContrasena={() => {
            setFallo(undefined);
            setEntrado(false);
            setModo('contrasena');
          }}
          olvidar={() => setOlvidar(true)}
        />
      )}

      {formulario && (
        <form
          onSubmit={(e) => {
            e.preventDefault();
            if (!listo || entrando) return;
            entrar(formulario === 'nueva' ? {usuario, contrasena, archivo} : {contrasena});
          }}
          style={{display: 'flex', flexDirection: 'column', gap: 12, maxWidth: ANCHO_CAMPOS}}>
          {formulario === 'nueva' && (
            <label style={{display: 'flex', flexDirection: 'column', gap: 5}}>
              <span style={ETIQUETA}>{t('clickedu.usuario')}</span>
              <TextInput
                value={usuario}
                onChange={(e) => setUsuario(e.target.value)}
                autoComplete="off"
                spellCheck={false}
                disabled={entrando}
              />
            </label>
          )}
          <CampoContrasena
            etiqueta={formulario === 'nueva' ? t('clickedu.contrasena') : t('clickedu.contrasena_nueva')}
            valor={contrasena}
            cambiar={setContrasena}
            disabled={entrando}
            enfocar={formulario === 'contrasena'}
          />
          {formulario === 'nueva' && (
            <div style={{display: 'flex', flexDirection: 'column', gap: 5}}>
              <span style={ETIQUETA}>{t('clickedu.archivo')}</span>
              <div style={{display: 'flex', alignItems: 'center', gap: 10, minWidth: 0}}>
                <span
                  style={{
                    flex: 1,
                    minWidth: 0,
                    overflow: 'hidden',
                    textOverflow: 'ellipsis',
                    whiteSpace: 'nowrap',
                    fontSize: 13,
                    fontFamily: archivo ? 'var(--font-mono)' : undefined,
                    color: archivo ? 'var(--text-primary)' : 'var(--text-tertiary)',
                  }}
                  title={archivo}>
                  {archivo ? nombreDe(archivo) : t('clickedu.archivo_ninguno')}
                </span>
                <Button
                  variant="ghost"
                  disabled={entrando}
                  onClick={async () => {
                    const ruta = await elegir();
                    if (ruta) setArchivo(ruta);
                  }}>
                  {t('clickedu.elegir_archivo')}
                </Button>
              </div>
            </div>
          )}
          <div style={{display: 'flex', flexWrap: 'wrap', alignItems: 'center', gap: 8}}>
            <Button type="submit" disabled={!listo || entrando} title={listo ? undefined : t('clickedu.falta_algo')}>
              {entrando ? t('clickedu.entrando') : t('clickedu.entrar')}
            </Button>
            {formulario === 'contrasena' && (
              <Button
                variant="ghost"
                disabled={entrando}
                onClick={() => {
                  setModo(undefined);
                  setContrasena('');
                  setFallo(undefined);
                }}>
                {t('clickedu.cancelar')}
              </Button>
            )}
          </div>
          {formulario === 'nueva' && (
            <p style={{margin: 0, fontSize: 12, lineHeight: 1.5, color: 'var(--text-tertiary)'}}>
              {t('clickedu.donde_se_guarda')}
            </p>
          )}
        </form>
      )}

      {entrado && (
        <p
          role="status"
          style={{
            margin: 0,
            display: 'flex',
            alignItems: 'center',
            gap: 6,
            fontSize: 13,
            color: encendido ? 'var(--success)' : 'var(--text-secondary)',
            transition: 'color 600ms',
          }}>
          <Icon d={ICONOS.comprobado} size={14} style={{flexShrink: 0}} />
          {t('clickedu.entrado')}
        </p>
      )}
      {fallo !== undefined && <Fallo fallo={fallo} />}

      <Modal open={olvidar} onClose={() => setOlvidar(false)} width={460} ariaLabel={t('clickedu.olvidar_titulo')}>
        <div style={{padding: '20px 22px'}}>
          <h2 style={{margin: 0, fontSize: 15, fontWeight: 600}}>{t('clickedu.olvidar_titulo')}</h2>
          <p style={{margin: '10px 0 0', fontSize: 13, lineHeight: 1.55, color: 'var(--text-secondary)'}}>
            {t('clickedu.olvidar_texto')}
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
          <Button variant="ghost" onClick={() => setOlvidar(false)}>
            {t('clickedu.cancelar')}
          </Button>
          <Button variant="stop" onClick={olvidarla}>
            {t('clickedu.olvidar_si')}
          </Button>
        </div>
      </Modal>
    </Card>
  );
}

const ETIQUETA: React.CSSProperties = {fontSize: 12.5, fontWeight: 500, color: 'var(--text-secondary)'};

function nombreDe(ruta: string) {
  return ruta.split(/[\\/]/).pop() ?? ruta;
}

/** La cuenta guardada: en gris, que es lo normal. */
function Guardada({
  cuenta,
  entrando,
  cambiarArchivo,
  cambiarContrasena,
  olvidar,
}: {
  cuenta: CuentaClickedu;
  entrando: boolean;
  cambiarArchivo: () => void;
  cambiarContrasena: () => void;
  olvidar: () => void;
}) {
  return (
    <>
      <div style={{display: 'flex', flexDirection: 'column', gap: 4, minWidth: 0}}>
        <div
          style={{
            fontFamily: 'var(--font-mono)',
            fontSize: 14,
            color: 'var(--text-primary)',
            overflowWrap: 'anywhere',
          }}>
          {cuenta.usuario}
        </div>
        {cuenta.archivo ? (
          <div style={{fontSize: 13, color: 'var(--text-secondary)', overflowWrap: 'anywhere'}}>
            {t('clickedu.archivo_guardado', {
              archivo: cuenta.archivo,
              fecha: cuenta.elegido ? new Date(cuenta.elegido * 1000).toLocaleDateString('es-ES') : '—',
            })}
          </div>
        ) : (
          <div role="alert" style={{fontSize: 13, color: 'var(--danger)'}}>
            {t('clickedu.archivo_falta')}
          </div>
        )}
      </div>
      <div style={{display: 'flex', flexWrap: 'wrap', alignItems: 'center', gap: 8}}>
        <Button variant="ghost" onClick={cambiarArchivo} disabled={entrando}>
          {t('clickedu.cambiar_archivo')}
        </Button>
        <Button variant="ghost" onClick={cambiarContrasena} disabled={entrando}>
          {t('clickedu.cambiar_contrasena')}
        </Button>
        <Button variant="ghost" onClick={olvidar} disabled={entrando}>
          {t('clickedu.olvidar')}
        </Button>
        {entrando && (
          <span role="status" style={{fontSize: 13, color: 'var(--accent-ink)'}}>
            {t('clickedu.entrando')}
          </span>
        )}
      </div>
    </>
  );
}

/** El campo de la contraseña, con el ojo para ver lo que se escribe. */
function CampoContrasena({
  etiqueta,
  valor,
  cambiar,
  disabled,
  enfocar,
}: {
  etiqueta: string;
  valor: string;
  cambiar: (v: string) => void;
  disabled: boolean;
  enfocar: boolean;
}) {
  const [ver, setVer] = useState(false);
  return (
    <div style={{display: 'flex', flexDirection: 'column', gap: 5}}>
      <label style={{display: 'flex', flexDirection: 'column', gap: 5}}>
        <span style={ETIQUETA}>{etiqueta}</span>
        <div style={{position: 'relative'}}>
          <TextInput
            type={ver ? 'text' : 'password'}
            value={valor}
            onChange={(e) => cambiar(e.target.value)}
            autoComplete="off"
            spellCheck={false}
            disabled={disabled}
            autoFocus={enfocar}
            style={{paddingRight: 36}}
          />
          <button
            type="button"
            className="boton-ojo"
            onClick={() => setVer((v) => !v)}
            aria-label={ver ? t('clickedu.esconder') : t('clickedu.ensenar')}
            aria-pressed={ver}
            title={ver ? t('clickedu.esconder') : t('clickedu.ensenar')}>
            <Icon d={ver ? ICONOS.ojoTachado : ICONOS.ojo} size={16} />
          </button>
        </div>
      </label>
    </div>
  );
}

/** El fallo, en rojo, con qué hacer. */
function Fallo({fallo}: {fallo: FalloClickedu | string}) {
  // Sin conexión, la causa de Windows, en pequeño: «se ha agotado el tiempo»
  // no es lo mismo que «no se encuentra el servidor».
  const detalle = typeof fallo === 'object' && fallo.tipo === 'sin_conexion' ? fallo.detalle : undefined;
  return (
    <div role="alert" style={{display: 'flex', flexDirection: 'column', gap: 4, maxWidth: 640}}>
      <p style={{margin: 0, fontSize: 13, lineHeight: 1.5, color: 'var(--danger)'}}>{mensaje(fallo)}</p>
      {detalle && (
        <p style={{margin: 0, fontSize: 12, color: 'var(--text-tertiary)', overflowWrap: 'anywhere'}}>{detalle}</p>
      )}
    </div>
  );
}

export function mensaje(fallo: FalloClickedu | string): string {
  if (typeof fallo === 'string') return fallo;
  switch (fallo.tipo) {
    case 'sin_conexion':
      return t('clickedu.fallo_sin_conexion');
    case 'servidor':
      return t('clickedu.fallo_servidor', {codigo: fallo.codigo});
    case 'usuario':
      return t('clickedu.fallo_usuario');
    case 'archivo':
      return t('clickedu.fallo_archivo');
    case 'no_entiendo':
      return t('clickedu.fallo_no_entiendo');
    case 'otro':
      return fallo.detalle;
  }
}
