/** @type {import('tailwindcss').Config} */
module.exports = {
  content: ["./gui/**/*.html", "./gui/**/*.js"],
  darkMode: 'class',
  theme: {
    extend: {
      screens: {
        'wide': '1100px',
      },
      fontFamily: { sans: ['Inter', 'sans-serif'] },
      transitionTimingFunction: {
        'ios': 'cubic-bezier(0.2, 0.8, 0.2, 1)',
      },
      colors: {
        brand: {
          50: '#eef9f9', 100: '#d4eff0', 200: '#aee0e2', 300: '#7bcad0',
          400: '#4cb0b7', 500: '#1a939a', 600: '#16797f', 700: '#136167',
          800: '#114f54', 900: '#0e4247'
        },
        zinc: {
          50: '#fafafa', 100: '#f4f4f5', 200: '#e4e4e7', 300: '#d4d4d8',
          400: '#a1a1aa', 500: '#71717a', 600: '#52525b', 700: '#3f3f46',
          800: '#27272a', 900: '#18181b', 950: '#09090b',
        }
      },
      animation: {
        'slide-up-fade': 'slide-up-fade 0.4s cubic-bezier(0.2, 0.8, 0.2, 1) forwards',
        'wave': 'wave 1.5s ease-in-out infinite',
        'spin-slow': 'spin 2s linear infinite',
        'pulse-ring': 'pulse-ring 1.5s cubic-bezier(0.215, 0.61, 0.355, 1) infinite',
        'shimmer': 'shimmer 2s infinite linear',
        'float': 'float 6s ease-in-out infinite',
      },
      keyframes: {
        'slide-up-fade': {
          '0%': { opacity: '0', transform: 'translateY(10px)' },
          '100%': { opacity: '1', transform: 'translateY(0)' }
        },
        'wave': {
          '0%, 100%': { transform: 'scaleY(0.5)' },
          '50%': { transform: 'scaleY(1.2)' }
        },
        'pulse-ring': {
          '0%': { transform: 'scale(0.33)' },
          '80%, 100%': { opacity: '0' }
        },
        'shimmer': {
          '100%': { transform: 'translateX(100%)' }
        },
        'float': {
          '0%, 100%': { transform: 'translateY(0)' },
          '50%': { transform: 'translateY(-10px)' },
        }
      }
    }
  },
  plugins: [],
}
