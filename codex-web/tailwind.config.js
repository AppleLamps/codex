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
          bg: 'var(--codex-bg)',
          surface: 'var(--codex-surface)',
          border: 'var(--codex-border)',
          text: 'var(--codex-text)',
          muted: 'var(--codex-muted)',
          accent: 'var(--codex-accent)',
          hover: 'var(--codex-hover)',
          // Semantic colors stay static
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
