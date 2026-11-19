/**
 * Ajustes. H1 trae el primero: **dónde están las plantillas**. Por defecto,
 * `Documentos\Gestor de Notas\plantillas` (D4); se puede elegir otra, y
 * volver a la de por defecto.
 *
 * **El micrófono** por el que se dicta (HUI): el de Windows, o el que se
 * elija, por ejemplo unos auriculares.
 *
 * Y **mejorar el dictado** (D8): lo guardado en el registro, el informe para
 * mandarlo y borrarlo.
 *
 * La cuenta de Clickedu llega con H4.
 */

import {invoke} from '@tauri-apps/api/core';
import {useState} from 'react';

import {Ruta} from '@components/clases/SinPlantillas';
import {TarjetaMicrofono} from '@components/dictado/TarjetaMicrofono';
import {Cabecera} from '@components/layout/Cabecera';
import {TarjetaRegistro} from '@components/registro/TarjetaRegistro';
import {Button} from '@components/ui/Button';
import {Card} from '@components/ui/Card';
import {Icon, ICONOS} from '@components/ui/Icon';
import {useClases} from '@modules/clases/useClases';
import {t} from '@modules/i18n';
import {useIdioma} from '@modules/i18n/useIdioma';

/** `visible`: la pantalla se está viendo (no se desmonta al irse). */
export function Ajustes({visible}: {visible: boolean}) {
  useIdioma();
  const {lista, error} = useClases();
  const [fallo, setFallo] = useState<string>();
  const carpeta = lista?.carpeta;

  // La lista se pone al día sola con el evento que manda el backend al
  // cambiar de carpeta; aquí solo queda decir si algo falla.
  const hacer = (cmd: string) => {
    setFallo(undefined);
    invoke(cmd).catch((e) => setFallo(String(e)));
  };

  return (
    <div style={{padding: '28px 32px'}}>
      <Cabecera titulo={t('ajustes.titulo')} />
      {/* El hueco entre tarjetas lo pone la pantalla, no cada tarjeta. */}
      <div style={{display: 'flex', flexDirection: 'column', gap: 16}}>
        <Card title={t('ajustes.plantillas')} subtitle={t('ajustes.plantillas_sub')}>
          {carpeta && (
            <>
              <div style={{display: 'flex', alignItems: 'baseline', gap: 10, minWidth: 0}}>
                <div style={{minWidth: 0, flex: 1}}>
                  <Ruta ruta={carpeta.ruta} />
                </div>
                {carpeta.porDefecto && (
                  <span style={{flexShrink: 0, fontSize: 11.5, color: 'var(--text-tertiary)'}}>
                    {t('ajustes.por_defecto')}
                  </span>
                )}
              </div>
              {!carpeta.existe && (
                <p role="alert" style={{margin: 0, fontSize: 13, color: 'var(--danger)'}}>
                  {t('clases.carpeta_no_existe')}
                </p>
              )}
              <div style={{display: 'flex', flexWrap: 'wrap', gap: 8}}>
                <Button variant="ghost" onClick={() => hacer('elegir_carpeta_plantillas')}>
                  {t('ajustes.cambiar')}
                </Button>
                {carpeta.existe && (
                  <Button variant="ghost" onClick={() => hacer('abrir_carpeta_plantillas')}>
                    <Icon d={ICONOS.carpeta} size={14} />
                    {t('clases.abrir_carpeta')}
                  </Button>
                )}
                {!carpeta.porDefecto && (
                  <Button variant="ghost" onClick={() => hacer('restablecer_carpeta_plantillas')}>
                    {t('ajustes.volver_por_defecto')}
                  </Button>
                )}
              </div>
            </>
          )}
          {(fallo ?? error) && (
            <p role="alert" style={{margin: 0, fontSize: 13, color: 'var(--danger)'}}>
              {fallo ?? error}
            </p>
          )}
        </Card>
        <TarjetaMicrofono visible={visible} />
        <TarjetaRegistro visible={visible} />
        <p style={{margin: 0, fontSize: 12.5, color: 'var(--text-tertiary)'}}>{t('ajustes.llegan')}</p>
      </div>
    </div>
  );
}
