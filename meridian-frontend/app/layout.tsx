import './globals.css';
import type { Metadata } from 'next';
import { Inter, Outfit, JetBrains_Mono } from 'next/font/google';
import { ThemeProvider } from 'next-themes';
import { Toaster } from 'sonner';
import { SkipToContent } from '@/components/SkipToContent';

const inter = Inter({
  subsets: ['latin'],
  variable: '--font-inter',
  display: 'swap',
});

const outfit = Outfit({
  subsets: ['latin'],
  variable: '--font-outfit',
  display: 'swap',
});

const jetbrainsMono = JetBrains_Mono({
  subsets: ['latin'],
  variable: '--font-jetbrains',
  display: 'swap',
});

export const metadata: Metadata = {
  title: 'Meridian | Sovereign Stablecoin Infrastructure',
  description: 'Turnkey infrastructure for launching compliant stablecoins backed by sovereign bonds.',
  manifest: '/site.webmanifest',
  icons: {
    icon: [
      { url: '/icon.svg', type: 'image/svg+xml' },
    ],
    apple: '/apple-icon.svg',
  },
  openGraph: {
    title: 'Meridian | Sovereign Stablecoin Infrastructure',
    description: 'Turnkey infrastructure for launching compliant stablecoins backed by sovereign bonds.',
    type: 'website',
  },
  twitter: {
    card: 'summary_large_image',
    title: 'Meridian | Sovereign Stablecoin Infrastructure',
    description: 'Turnkey infrastructure for launching compliant stablecoins backed by sovereign bonds.',
  },
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="en" suppressHydrationWarning>
      <body className={`${inter.variable} ${outfit.variable} ${jetbrainsMono.variable} font-sans bg-background-dark text-text-primary antialiased selection:bg-primary selection:text-background-dark`}>
        <ThemeProvider attribute="class" defaultTheme="dark" enableSystem={false}>
          <SkipToContent />
          <main id="main-content" tabIndex={-1} className="outline-none">
            {children}
          </main>
          <Toaster
            theme="dark"
            position="top-right"
            toastOptions={{
              style: {
                background: '#141416',
                border: '1px solid #27272A',
                color: '#FAFAFA',
              },
            }}
          />
        </ThemeProvider>
      </body>
    </html>
  );
}
