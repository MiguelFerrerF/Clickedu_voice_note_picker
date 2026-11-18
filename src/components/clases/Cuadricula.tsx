/**
 * La cuadrícula de alumnos de una clase, **en lectura** (H1). En H2 la nota
 * pasa a ser un campo; aquí es texto.
 *
 * La regla de color (D6): sin nota en gris, una nota es texto normal (también
 * un suspenso), y solo lo que **no es nota** va en rojo.
 */

import {t} from '@modules/i18n';
import type {Alumno, Nota} from '@modules/clases/tipos';

const COLUMNAS = '52px minmax(200px, 1fr) 96px';

export function Cuadricula({alumnos}: {alumnos: Alumno[]}) {
  return (
    <div style={{border: '1px solid var(--border)', borderRadius: 8, overflow: 'hidden', background: 'var(--bg-card)'}}>
      <div
        role="row"
        style={{
          display: 'grid',
          gridTemplateColumns: COLUMNAS,
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
        <span>{t('cuadricula.n')}</span>
        <span>{t('cuadricula.alumno')}</span>
        <span style={{textAlign: 'right', paddingRight: 8}}>{t('cuadricula.nota')}</span>
      </div>
      {alumnos.map((a, i) => (
        <div
          key={`${a.fila}`}
          role="row"
          style={{
            display: 'grid',
            gridTemplateColumns: COLUMNAS,
            alignItems: 'center',
            minHeight: 'var(--row-h)',
            padding: '4px var(--row-pad)',
            borderBottom: i < alumnos.length - 1 ? '1px solid var(--border)' : 'none',
          }}>
          <span style={{fontFamily: 'var(--font-mono)', fontSize: 12, color: 'var(--text-tertiary)'}}>{i + 1}</span>
          <span
            style={{
              fontSize: 13,
              color: a.nombre.trim() ? 'var(--text-primary)' : 'var(--text-tertiary)',
              overflow: 'hidden',
              textOverflow: 'ellipsis',
              whiteSpace: 'nowrap',
            }}
            title={a.nombre.trim() || undefined}>
            {a.nombre.trim() ? a.nombre : t('cuadricula.sin_nombre', {id: a.id})}
          </span>
          <CeldaNota nota={a.nota} />
        </div>
      ))}
    </div>
  );
}

function CeldaNota({nota}: {nota: Nota}) {
  const base: React.CSSProperties = {
    textAlign: 'right',
    paddingRight: 8,
    fontFamily: 'var(--font-mono)',
    fontSize: 13,
    whiteSpace: 'nowrap',
    overflow: 'hidden',
    textOverflow: 'ellipsis',
  };
  switch (nota.tipo) {
    case 'vacia':
      return <span style={{...base, color: 'var(--text-tertiary)'}}>—</span>;
    case 'valor':
      return <span style={{...base, color: 'var(--text-primary)'}}>{String(nota.valor)}</span>;
    case 'invalida':
      return (
        <span style={{...base, color: 'var(--danger)', fontWeight: 600}} title={t('cuadricula.no_es_nota')}>
          {nota.valor}
        </span>
      );
  }
}
