/**
 * Icono SVG en línea, sin dependencias.
 *
 * COPIADO de library-app (y esta, de la Quality Test App): acepta un `d`
 * suelto o un array, para que un icono de varios trazos salga en un solo
 * elemento. El color sigue a `currentColor`, así que se tiñe desde el
 * contenedor.
 */

export interface IconProps {
  d: string | readonly string[];
  size?: number;
  strokeWidth?: number;
  className?: string;
  style?: React.CSSProperties;
}

export function Icon({
  d,
  size = 16,
  strokeWidth = 1.75,
  className,
  style,
}: IconProps) {
  return (
    <svg
      width={size}
      height={size}
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth={strokeWidth}
      strokeLinecap="round"
      strokeLinejoin="round"
      className={className}
      style={style}
      aria-hidden="true">
      {Array.isArray(d) ? (
        (d as readonly string[]).map((p, i) => <path key={i} d={p} />)
      ) : (
        <path d={d as string} />
      )}
    </svg>
  );
}

/**
 * Catálogo de iconos del Gestor de Notas.
 *
 * El trazo es el de library-app —24×24, extremos redondos, 1.75 por
 * defecto— para que la familia sea la misma. Los dibujos son los de Lucide
 * (licencia ISC), copiados a mano: solo los que se usan, sin la dependencia.
 */
export const ICONOS = {
  // ── Aula ──
  // Un portapapeles con visto: poner notas.
  calificar: [
    'M9 5H7a2 2 0 0 0-2 2v12a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2V7a2 2 0 0 0-2-2h-2',
    'M9 3h6v4H9z',
    'M9 14l2 2 4-4',
  ],
  clases: [
    'M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2',
    'M9 11a4 4 0 1 0 0-8 4 4 0 0 0 0 8z',
    'M22 21v-2a4 4 0 0 0-3-3.87',
    'M16 3.13a4 4 0 0 1 0 7.75',
  ],

  // ── Clickedu ──
  clickedu: [
    'M4 14.9A7 7 0 1 1 15.7 8h1.8a4.5 4.5 0 0 1 2.5 8.2',
    'M12 12v9',
    'M8 16l4-4 4 4',
  ],

  // ── Análisis ──
  estadisticas: ['M3 3v18h18', 'M7 16v-5M12 16V8M17 16v-4'],

  // ── Ajustes ──
  ajustes: [
    'M21 4h-7M10 4H3M21 12h-9M8 12H3M21 20h-5M12 20H3',
    'M14 2v4M8 10v4M16 18v4',
  ],
  // Un «?» en un círculo: la Ayuda (H9).
  ayuda: [
    'M12 22a10 10 0 1 0 0-20 10 10 0 0 0 0 20z',
    'M9.09 9a3 3 0 0 1 5.83 1c0 2-3 3-3 3',
    'M12 17h.01',
  ],
  muestrario: [
    'M12 22a10 10 0 1 1 10-10c0 2.5-2 3.5-4 3.5h-2a2 2 0 0 0-1.5 3.3c.4.5.5 1 .5 1.5 0 1-.9 1.7-3 1.7z',
    'M7.5 10.5h.01M10.5 7.5h.01M15.5 8.5h.01',
  ],

  // ── Voz ──
  microfono: [
    'M12 2a3 3 0 0 0-3 3v7a3 3 0 0 0 6 0V5a3 3 0 0 0-3-3z',
    'M19 10v2a7 7 0 0 1-14 0v-2',
    'M12 19v3',
  ],

  // ── Utilidad ──
  carpeta: 'M3 7a2 2 0 0 1 2-2h4l2 2h8a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z',
  exportar: 'M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4M17 8l-5-5-5 5M12 3v12',
  buscar: 'M21 21l-4.35-4.35M11 19a8 8 0 1 1 0-16 8 8 0 0 1 0 16z',
  info: ['M12 22a10 10 0 1 0 0-20 10 10 0 0 0 0 20z', 'M12 16v-4M12 8h.01'],
  aviso: [
    'M10.3 3.9L1.8 18a2 2 0 0 0 1.7 3h17a2 2 0 0 0 1.7-3L13.7 3.9a2 2 0 0 0-3.4 0z',
    'M12 9v4M12 17h.01',
  ],
  cerrar: 'M18 6L6 18M6 6l12 12',
  mas: 'M12 5v14M5 12h14',
  comprobado: 'M20 6L9 17l-5-5',
  // El ojo del campo de la contraseña (H4): enseñarla y esconderla.
  ojo: ['M2 12s3-7 10-7 10 7 10 7-3 7-10 7-10-7-10-7z', 'M12 15a3 3 0 1 0 0-6 3 3 0 0 0 0 6z'],
  ojoTachado: [
    'M9.9 4.24A9.12 9.12 0 0 1 12 4c7 0 10 7 10 7a18.5 18.5 0 0 1-2.16 3.19',
    'M14.12 14.12a3 3 0 1 1-4.24-4.24',
    'M17.94 17.94A10.07 10.07 0 0 1 12 19c-7 0-10-7-10-7a18.45 18.45 0 0 1 5.06-5.94',
    'M2 2l20 20',
  ],
  galonIzquierda: 'M15 6l-6 6 6 6',
  galonDerecha: 'M9 6l6 6-6 6',
  galonAbajo: 'M6 9l6 6 6-6',
} as const;

export type ClaveIcono = keyof typeof ICONOS;
