/**
 * Lo que devuelven los comandos de Clickedu (`src-tauri/src/commands/clickedu.rs`).
 */

/** La cuenta guardada, **sin la contraseña**. */
export interface CuentaClickedu {
  usuario: string;
  /** El nombre del archivo de seguridad; `null` si falta. */
  archivo: string | null;
  /** Cuándo se eligió el archivo, en segundos desde 1970. */
  elegido: number | null;
}

/** Por qué no se ha podido entrar (`clickedu::cliente::Fallo`). */
export type FalloClickedu =
  | {tipo: 'sin_conexion'; detalle: string}
  | {tipo: 'servidor'; codigo: number}
  | {tipo: 'usuario'}
  | {tipo: 'archivo'}
  | {tipo: 'no_entiendo'; paso: string}
  | {tipo: 'otro'; detalle: string};

// ───────────────────────── Traer las plantillas (H5) ─────────────────────────

/** Cómo se ha leído algo de Clickedu: degradado es el aviso temprano. */
export type EstadoLectura = {estado: 'bien'} | {estado: 'degradado'; motivo: string};

/** Clickedu ha cambiado y una ficha no se lee (`fichas::Roto`). */
export interface Roto {
  ficha: string;
  motivo: string;
}

/** Una materia, tal cual la pone Clickedu. */
export interface Materia {
  id: string;
  /** «Física y Química». */
  nombre: string;
  /** «3º E.S.O.-A». */
  grupo: string;
  /** «Tercero de ESO». */
  curso: string | null;
}

export interface Retoque {
  de: string;
  a: string;
}

export type AvisoNombre = {tipo: 'curso_tal_cual'; encabezado: string} | {tipo: 'sin_curso'};

export type FalloBajada =
  | {tipo: 'evaluaciones'; roto: Roto}
  | {tipo: 'no_es_excel'; estado: number; contenido: string};

export type FalloMateria = {tipo: 'bajar'; fallo: FalloBajada} | {tipo: 'no_se_lee'; error: string};

export type EstadoMateria =
  | {tipo: 'nueva'; alumnos: number}
  | {tipo: 'igual'; alumnos: number}
  | {tipo: 'cambia'; alumnos: number}
  | {tipo: 'vacia'}
  | {tipo: 'fallo'; fallo: FalloMateria};

export interface PlanMateria {
  materia: Materia;
  fichero: string;
  /** De otro curso: va aparte, en «Pendientes». */
  pendiente: boolean;
  retoques: Retoque[];
  avisos: AvisoNombre[];
  estado: EstadoMateria;
  estado_evaluaciones: EstadoLectura | null;
}

/** Por qué pasan las notas a esa: lo recordado, el nombre, los alumnos o la respuesta. */
export type Por = 'recordada' | 'mismo_nombre' | 'alumnos' | 'respuesta';

export interface Paso {
  a: string;
  por: Por;
  pasan: number;
  /** Con nota y ya no están en Clickedu. */
  sin_alumno: number;
  /** Con nota y sin un `Id` que sirva. */
  sin_id: number;
}

/** Lo que va a `antiguas`. */
export interface Apartada {
  fichero: string;
  notas: Paso | null;
  /** Con nota y no pasan a ninguna: lo ha respondido el usuario. */
  descartadas: number;
}

/** Una vieja con notas que no se sabe a qué nueva pasan. */
export interface Duda {
  fichero: string;
  /** Cuántos alumnos con nota. */
  notas: number;
  candidatas: string[];
}

export interface Plan {
  materias: PlanMateria[];
  apartadas: Apartada[];
  dudas: Duda[];
}

export type FalloAplicar =
  | {tipo: 'mover'; fichero: string; error: string}
  | {tipo: 'escribir'; fichero: string; error: string | null}
  | {tipo: 'notas'; fichero: string; a: string}
  | {tipo: 'guardar'; error: string};

export interface Hecho {
  escritas: string[];
  /** `a`, desde la carpeta de plantillas: `antiguas\2026-11-22\…`. */
  movidas: {fichero: string; a: string}[];
  pasan: number;
  fallos: FalloAplicar[];
}

export interface Traer {
  plan: Plan;
  /** `null`: espera a que se confirme. */
  hecho: Hecho | null;
  primeraVez: boolean;
  estadoMaterias: EstadoLectura | null;
  /** Lo que ha cambiado en la forma de las páginas, sin nombres. */
  cambios: string[];
}

export interface AvanceTraer {
  hechas: number;
  total: number;
}

/** La respuesta a una vieja con notas. */
export type Respuesta = {tipo: 'pasar'; a: string} | {tipo: 'ninguna'};

/** Por qué no se ha podido traer nada (`clickedu::traer::FalloTraer`). */
export type FalloTraer =
  | {tipo: 'conexion'; fallo: FalloClickedu}
  | {tipo: 'materias'; roto: Roto}
  | {tipo: 'sin_entrar'}
  | {tipo: 'otro'; detalle: string};

// ───────────────────────── Subir las notas (H6) ─────────────────────────

/** En qué paso va la subida (`clickedu::subir::Paso`). */
export type PasoSubir = 'leer' | 'subir' | 'importar' | 'comprobar';

/** Por qué no se ha subido (`clickedu::subir::FalloSubir`). */
export type FalloSubir =
  | {tipo: 'conexion'; paso: PasoSubir; fallo: FalloClickedu}
  | {tipo: 'sin_entrar'}
  | {tipo: 'sin_item'}
  | {tipo: 'cambiado'; paso: PasoSubir; motivo: string}
  | {tipo: 'otro'; detalle: string};

/** Una nota que se pisaría (`commands::subir::Cambio`). */
export interface Cambio {
  nombre: string;
  /** La de Clickedu, tal cual. */
  antes: string;
  nueva: number;
}

/** Lo que se va a subir, recién leído Clickedu (`commands::subir::Previa`). */
export interface PreviaSubir {
  item: string;
  evaluacion: string;
  alumnos: number;
  conNota: number;
  /** Sin nota en Clickedu. */
  nuevas: number;
  /** La misma en Clickedu. */
  iguales: number;
  /** **Se pisarían.** */
  cambiarian: Cambio[];
  /** El Excel exportado: el plan B. */
  excel: string;
}

/** Cómo ha ido (`commands::subir::Subido`). */
export interface Subido {
  /** Comprobadas en Clickedu. */
  entraron: number;
  /** Los alumnos cuya nota no está en Clickedu tras importar. */
  noEntraron: string[];
  avisos: string[];
}
