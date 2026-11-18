/**
 * Punto de entrada de Vite/React.
 *
 * Arranca la raíz de React, aplica los estilos globales y monta <App/>.
 * Toda la composición de la app vive en `App.tsx`.
 */

import React from 'react';
import ReactDOM from 'react-dom/client';

// Las fuentes primero: `globals.css` ya usa var(--font-body).
import './styles/fonts.css';
import './styles/globals.css';
import App from './App';

const container = document.getElementById('root');
if (!container) {
  throw new Error('No encuentro el contenedor #root — revisa index.html');
}

ReactDOM.createRoot(container).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
