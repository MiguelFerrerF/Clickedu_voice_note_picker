/**
 * Las notas que se escriben a mano (H2): qué es una nota y cuál vale.
 *
 * Una nota va de 0 a 10 con dos decimales como mucho, con coma o con punto
 * (`7,5` es `7.5`, que es como la acepta Clickedu). Lo que no lo es **no se
 * guarda ni se corrige**: se queda en rojo en su campo, diciendo por qué. El
 * backend valida lo mismo (`sesion::validar`); esto es para decirlo al
 * momento.
 */

import type {Alumno, Nota, Puestas} from '@modules/clases/tipos';

export type MotivoMal = 'no_es_numero' | 'fuera_de_rango' | 'decimales';

/** Lo escrito en un campo de nota, leído. */
export type Escrita = {tipo: 'vacia'} | {tipo: 'valor'; valor: number} | {tipo: 'mal'; motivo: MotivoMal};

export function leerEscrita(texto: string): Escrita {
  const s = texto.trim().replace(',', '.');
  if (!s) return {tipo: 'vacia'};
  if (!/^(\d+(\.\d*)?|\.\d+)$/.test(s)) return {tipo: 'mal', motivo: 'no_es_numero'};
  const valor = Number(s);
  if (!(valor >= 0 && valor <= 10)) return {tipo: 'mal', motivo: 'fuera_de_rango'};
  // `7.50` lleva dos decimales; `7.500`, también: los ceros de la cola no cuentan.
  const decimales = (s.split('.')[1] ?? '').replace(/0+$/, '');
  if (decimales.length > 2) return {tipo: 'mal', motivo: 'decimales'};
  return {tipo: 'valor', valor};
}

/** La nota que vale: la puesta en la sesión si la hay, y si no la de la plantilla. */
export function notaDe(alumno: Alumno, puestas: Puestas): Nota {
  const puesta = puestas[alumno.clave];
  if (puesta === undefined) return alumno.nota;
  return puesta === null ? {tipo: 'vacia'} : {tipo: 'valor', valor: puesta};
}

/** Como se enseña: con punto y sin ceros de más (`7.5`, `7`), como se exporta. */
export function notaComoTexto(nota: Nota): string {
  switch (nota.tipo) {
    case 'vacia':
      return '';
    case 'valor':
      return String(nota.valor);
    case 'invalida':
      return nota.valor;
  }
}
