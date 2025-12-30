import type { Metadata } from 'next';
import './globals.css';

export const metadata: Metadata = {
  title: 'Codex Web UI',
  description: 'Web interface for OpenAI Codex CLI',
  icons: {
    icon: '/favicon.ico',
  },
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="en" className="dark">
      <body className="bg-codex-bg text-codex-text antialiased">
        {children}
      </body>
    </html>
  );
}
