/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    './src/pages/**/*.{js,ts,jsx,tsx,mdx}',
    './src/components/**/*.{js,ts,jsx,tsx,mdx}',
    './src/app/**/*.{js,ts,jsx,tsx,mdx}',
  ],
  theme: {
    extend: {
      colors: {
        codex: {
          bg: '#0a0a0a',
          surface: '#141414',
          border: '#262626',
          text: '#fafafa',
          muted: '#a1a1a1',
          accent: '#ffffff',
          hover: '#1f1f1f',
          success: '#22c55e',
          error: '#ef4444',
          warning: '#f59e0b',
        },
      },
      fontFamily: {
        mono: ['ui-monospace', 'SFMono-Regular', 'Menlo', 'Monaco', 'Consolas', 'monospace'],
      },
    },
  },
  plugins: [],
};
