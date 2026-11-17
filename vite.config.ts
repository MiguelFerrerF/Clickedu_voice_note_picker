import react from '@vitejs/plugin-react';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import {defineConfig} from 'vite';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const src = (p: string) => path.resolve(__dirname, 'src', p);

// Tauri espera un puerto fijo y falla si no está libre.
const host = process.env.TAURI_DEV_HOST;

export default defineConfig(
    async () => ({
      plugins: [react()],

      resolve: {
        // Mantener sincronizado con `tsconfig.json::compilerOptions.paths`.
        alias: {
          '@': path.resolve(__dirname, 'src'),
          '@components': src('components'),
          '@screens': src('screens'),
          '@features': src('features'),
          '@modules': src('modules'),
          '@styles': src('styles'),
        },
      },

      clearScreen: false,
      server: {
        port: 1420,
        strictPort: true,
        host: host || false,
        hmr: host ? {protocol: 'ws', host, port: 1421} : undefined,
        watch: {
          // No vigilar `src-tauri`: si no, Vite se reinicia con cada cambio
          // en Rust.
          ignored: ['**/src-tauri/**'],
        },
      },

      envPrefix: ['VITE_', 'TAURI_ENV_*'],

      build: {
        // Tauri 2 apunta a Chromium ≥ 105.
        target: 'chrome105',
        minify: !process.env.TAURI_ENV_DEBUG ? 'esbuild' : false,
        sourcemap: !!process.env.TAURI_ENV_DEBUG,
      },
    }));
