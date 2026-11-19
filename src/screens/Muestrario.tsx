/**
 * Muestrario — el catálogo de componentes con la paleta, para ver de un
 * vistazo que un cambio de token no rompe nada.
 *
 * **Solo en desarrollo** (la barra lateral no lo enseña en un build). Sirvió
 * para elegir la familia de neutros sobre pantalla (`docs/02-sistema-visual.md`).
 *
 * La cuadrícula de abajo es una **maqueta**: no es el componente de H1, solo
 * los seis estados de una nota pintados con la regla de color, para decidir
 * si se leen bien antes de construir nada.
 */

import {useState} from 'react';

import logo from '@/assets/logo.png';
import {Modal} from '@components/dialogs/Modal';
import {Cabecera} from '@components/layout/Cabecera';
import {Button} from '@components/ui/Button';
import {Card} from '@components/ui/Card';
import {Dropdown} from '@components/ui/Dropdown';
import {Icon, ICONOS} from '@components/ui/Icon';
import {Progreso} from '@components/ui/Progreso';
import {Select} from '@components/ui/Select';
import {NumInput, TextInput} from '@components/ui/TextInput';
import {t} from '@modules/i18n';
import styles from '@styles/cuadricula.module.css';
import {useIdioma} from '@modules/i18n/useIdioma';

// ─────────────────────────────────────────────────────────────────────────────
// La pantalla
// ─────────────────────────────────────────────────────────────────────────────

export function Muestrario() {
  useIdioma();
  const [dialogo, setDialogo] = useState(false);
  const [clase, setClase] = useState('mat3a');

  return (
    <div style={{padding: '28px 32px', maxWidth: 1040}}>
      <Cabecera titulo={t('muestrario.titulo')} intro={t('muestrario.subtitulo')} />

      <div style={{display: 'flex', flexDirection: 'column', gap: 16}}>
        <Card title={t('muestrario.cuadricula')} subtitle={t('muestrario.cuadricula_sub')}>
          <Cuadricula />
        </Card>

        <div style={{display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(320px, 1fr))', gap: 16}}>
          <Card title={t('muestrario.botones')}>
            <div style={{display: 'flex', flexWrap: 'wrap', gap: 8}}>
              <Button onClick={() => setDialogo(true)}>
                <Icon d={ICONOS.clickedu} size={14} />
                {t('muestrario.boton_primario')}
              </Button>
              <Button variant="ghost">
                <Icon d={ICONOS.exportar} size={14} />
                {t('muestrario.boton_secundario')}
              </Button>
              <Button variant="stop">{t('muestrario.boton_peligro')}</Button>
              <Button disabled>{t('muestrario.boton_desactivado')}</Button>
              <Button variant="ghost" disabled>
                {t('muestrario.boton_desactivado')}
              </Button>
            </div>
            <div>
              <Button variant="ghost" onClick={() => setDialogo(true)}>
                {t('muestrario.abrir_dialogo')}
              </Button>
            </div>
          </Card>

          <Card title={t('muestrario.campos')}>
            <TextInput placeholder={t('muestrario.campo_texto')} />
            <div style={{display: 'grid', gridTemplateColumns: '1fr 96px', gap: 8}}>
              <Select
                defaultValue="mat3a"
                options={[
                  {value: 'mat3a', label: 'Matemáticas 3.º A ESO'},
                  {value: 'bio1', label: 'Biología 1.º AB Bachillerato'},
                ]}
              />
              <NumInput defaultValue={7.5} step={0.1} />
            </div>
            <div>
              <Etiqueta>{t('muestrario.campo_clase')}</Etiqueta>
              <Dropdown
                value={clase}
                onChange={setClase}
                options={[
                  {value: 'mat3a', label: 'Matemáticas 3.º A ESO'},
                  {value: 'mat3b', label: 'Matemáticas 3.º B ESO'},
                  {value: 'bio1', label: 'Biología 1.º AB Bachillerato'},
                  {value: 'fis2', label: 'Física 2.º Bachillerato'},
                ]}
              />
            </div>
          </Card>

          <Card title={t('muestrario.progreso')}>
            <Progreso rotulo={t('muestrario.progreso_subiendo')} hecho={14} total={24} />
            <Progreso rotulo={t('muestrario.progreso_conectando')} />
          </Card>

          <Card title={t('muestrario.colores')}>
            <Colores />
          </Card>
        </div>
      </div>

      <Modal open={dialogo} onClose={() => setDialogo(false)} width={460} ariaLabel={t('muestrario.dialogo_titulo')}>
        <div style={{padding: '20px 22px'}}>
          <h2 style={{margin: 0, fontSize: 15, fontWeight: 600}}>{t('muestrario.dialogo_titulo')}</h2>
          <p style={{margin: '10px 0 0', fontSize: 13, lineHeight: 1.55, color: 'var(--text-secondary)'}}>
            {t('muestrario.dialogo_texto')}
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
          <Button variant="ghost" onClick={() => setDialogo(false)}>
            {t('muestrario.cancelar')}
          </Button>
          <Button onClick={() => setDialogo(false)}>{t('muestrario.boton_primario')}</Button>
        </div>
      </Modal>
    </div>
  );
}

// ─────────────────────────────────────────────────────────────────────────────
// Piezas
// ─────────────────────────────────────────────────────────────────────────────

function Etiqueta({children}: {children: React.ReactNode}) {
  return (
    <div
      style={{
        fontSize: 10.5,
        fontWeight: 600,
        letterSpacing: 1,
        textTransform: 'uppercase',
        color: 'var(--text-tertiary)',
        marginBottom: 6,
      }}>
      {children}
    </div>
  );
}

/** Los tokens de color, cada uno con su muestra. */
function Colores() {
  const grupos: [string, string[]][] = [
    ['Superficies', ['--bg', '--bg-card', '--bg-sunk', '--bg-active', '--border', '--border-strong']],
    ['Texto', ['--text-primary', '--text-secondary', '--text-tertiary']],
    ['Acento', ['--accent', '--accent-ink', '--accent-soft', '--accent-wash']],
    ['Semánticos', ['--success', '--danger', '--warn']],
  ];
  return (
    <div style={{display: 'flex', flexDirection: 'column', gap: 12}}>
      <div style={{display: 'flex', alignItems: 'center', gap: 10, fontSize: 12, color: 'var(--text-secondary)'}}>
        <img src={logo} alt="" width={26} height={26} style={{objectFit: 'contain'}} />
        <span>
          Logo <code style={{fontFamily: 'var(--font-mono)', fontSize: 11.5}}>#0F9096</code> frente a{' '}
          <code style={{fontFamily: 'var(--font-mono)', fontSize: 11.5, color: 'var(--accent-ink)'}}>--accent</code>
        </span>
        <span style={{width: 26, height: 26, borderRadius: 6, background: 'var(--accent)'}} />
      </div>
      {grupos.map(([nombre, tokens]) => (
        <div key={nombre}>
          <div style={{fontSize: 11, color: 'var(--text-tertiary)', marginBottom: 6}}>{nombre}</div>
          <div style={{display: 'flex', flexWrap: 'wrap', gap: 6}}>
            {tokens.map((tk) => (
              <div key={tk} title={tk} style={{width: 64}}>
                <div
                  style={{
                    height: 32,
                    borderRadius: 6,
                    background: `var(${tk})`,
                    border: '1px solid var(--border)',
                  }}
                />
                <div
                  style={{
                    marginTop: 4,
                    fontFamily: 'var(--font-mono)',
                    fontSize: 9.5,
                    color: 'var(--text-tertiary)',
                    overflow: 'hidden',
                    textOverflow: 'ellipsis',
                    whiteSpace: 'nowrap',
                  }}>
                  {tk.slice(2)}
                </div>
              </div>
            ))}
          </div>
        </div>
      ))}
    </div>
  );
}

// ── La maqueta de la cuadrícula ──

type EstadoNota = 'sin_nota' | 'nota' | 'suspenso' | 'dictada' | 'dudosa' | 'invalida';

const FILAS: {n: number; alumno: string; nota: string; estado: EstadoNota}[] = [
  {n: 1, alumno: 'Castro Molina, Lucía', nota: '', estado: 'sin_nota'},
  {n: 2, alumno: 'Martínez Ruiz, Hugo', nota: '8.7', estado: 'nota'},
  {n: 3, alumno: 'Ramos Ortega, Martina', nota: '3.4', estado: 'suspenso'},
  {n: 4, alumno: 'Navarro Gil, Mateo', nota: '7.5', estado: 'dictada'},
  {n: 5, alumno: 'Pérez Gómez, Irene', nota: '', estado: 'dudosa'},
  {n: 6, alumno: 'Ionescu, Ștefan', nota: '12', estado: 'invalida'},
];

function Cuadricula() {
  return (
    <div style={{border: '1px solid var(--border)', borderRadius: 8, overflow: 'hidden'}}>
      <div
        style={{
          display: 'grid',
          gridTemplateColumns: '44px minmax(180px, 1fr) 96px minmax(200px, 1.2fr)',
          alignItems: 'center',
          height: 32,
          padding: '0 var(--row-pad)',
          background: 'var(--bg-sunk)',
          borderBottom: '1px solid var(--border)',
          fontSize: 10.5,
          fontWeight: 600,
          letterSpacing: 0.8,
          textTransform: 'uppercase',
          color: 'var(--text-tertiary)',
        }}>
        <span>Nº</span>
        <span>{t('muestrario.col_alumno')}</span>
        <span style={{textAlign: 'right', paddingRight: 8}}>{t('muestrario.col_nota')}</span>
        <span style={{paddingLeft: 16}}>{t('muestrario.col_estado')}</span>
      </div>
      {FILAS.map((f, i) => (
        <div
          key={f.n}
          style={{
            display: 'grid',
            gridTemplateColumns: '44px minmax(180px, 1fr) 96px minmax(200px, 1.2fr)',
            alignItems: 'center',
            minHeight: 'var(--row-h)',
            padding: '4px var(--row-pad)',
            borderBottom: i < FILAS.length - 1 ? '1px solid var(--border)' : 'none',
            background: 'var(--bg-card)',
          }}>
          <span style={{fontFamily: 'var(--font-mono)', fontSize: 12, color: 'var(--text-tertiary)'}}>{f.n}</span>
          <span style={{fontSize: 13, color: 'var(--text-primary)'}}>{f.alumno}</span>
          <CeldaNota nota={f.nota} estado={f.estado} />
          <span style={{paddingLeft: 16, fontSize: 12, color: 'var(--text-secondary)'}}>
            {f.estado === 'dudosa' ? <Confirmar /> : t(`muestrario.estado_${f.estado}`)}
          </span>
        </div>
      ))}
    </div>
  );
}

function CeldaNota({nota, estado}: {nota: string; estado: EstadoNota}) {
  const base: React.CSSProperties = {
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'flex-end',
    gap: 6,
    height: 28,
    padding: '0 8px',
    borderRadius: 6,
    border: '1px solid var(--border-strong)',
    background: 'var(--bg-card)',
    fontFamily: 'var(--font-mono)',
    fontSize: 13,
    color: 'var(--text-primary)',
  };
  const porEstado: Record<EstadoNota, React.CSSProperties> = {
    sin_nota: {color: 'var(--text-tertiary)'},
    nota: {},
    suspenso: {},
    dictada: {
      borderColor: 'var(--success)',
      background: 'color-mix(in oklab, var(--success) 10%, var(--bg-card))',
      color: 'var(--success)',
      fontWeight: 600,
    },
    // Su nota no cambia hasta que se elige: lo propuesto va al lado (H3b).
    dudosa: {color: 'var(--text-tertiary)'},
    invalida: {
      borderColor: 'var(--danger)',
      color: 'var(--danger)',
      boxShadow: '0 0 0 3px color-mix(in oklch, var(--danger) 12%, transparent)',
    },
  };
  return (
    <div style={{...base, ...porEstado[estado]}}>
      {estado === 'dictada' && <Icon d={ICONOS.microfono} size={12} />}
      {nota || '—'}
    </div>
  );
}

/** Lo que propone un dictado dudoso, como en la cuadrícula de Calificar. */
function Confirmar() {
  return (
    <span style={{display: 'inline-flex', alignItems: 'center', gap: 8}}>
      <button type="button" className={styles.propuesta}>
        {t('cuadricula.propuesta', {nota: '6.8'})}
      </button>
      {t('muestrario.estado_dudosa')}
    </span>
  );
}
