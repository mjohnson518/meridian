import type { Metadata } from "next";
import { Inter, IBM_Plex_Mono } from "next/font/google";
import "./globals.css";

const inter = Inter({
  subsets: ["latin"],
  variable: "--font-sans",
  display: "swap",
});

const ibmPlexMono = IBM_Plex_Mono({
  weight: ["400", "500", "600"],
  subsets: ["latin"],
  variable: "--font-mono",
  display: "swap",
});

export const metadata: Metadata = {
  title: "Meridian Finance - Multi-Currency Stablecoin Platform",
  description: "Professional-grade infrastructure for multi-currency stablecoins backed by sovereign bonds",
  keywords: "stablecoin, multi-currency, EUR, GBP, JPY, sovereign bonds, DeFi",
  authors: [{ name: "Meridian Finance" }],
  openGraph: {
    title: "Meridian Finance",
    description: "Multi-Currency Stablecoin Platform",
    type: "website",
  },
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en" className={`${inter.variable} ${ibmPlexMono.variable}`}>
      <body className="min-h-screen bg-sacred-white text-sacred-black antialiased">
        {/* Header */}
        <header className="border-b border-sacred-gray-200">
          <div className="sacred-container">
            <nav className="flex items-center justify-between h-16">
              <div className="flex items-center space-x-8">
                <a href="/" className="font-mono text-lg font-medium tracking-tight no-underline hover:opacity-70">
                  MERIDIAN
                </a>
                <div className="hidden md:flex items-center space-x-6">
                  <a 
                    href="/reserves" 
                    className="text-sm font-mono uppercase tracking-wider text-sacred-gray-600 hover:text-sacred-black transition-colors no-underline"
                  >
                    Reserves
                  </a>
                  <a 
                    href="/app" 
                    className="text-sm font-mono uppercase tracking-wider text-sacred-gray-600 hover:text-sacred-black transition-colors no-underline"
                  >
                    App
                  </a>
                  <a 
                    href="/developers" 
                    className="text-sm font-mono uppercase tracking-wider text-sacred-gray-600 hover:text-sacred-black transition-colors no-underline"
                  >
                    Developers
                  </a>
                  <a 
                    href="/docs" 
                    className="text-sm font-mono uppercase tracking-wider text-sacred-gray-600 hover:text-sacred-black transition-colors no-underline"
                  >
                    Docs
                  </a>
                </div>
              </div>
              
              <div className="flex items-center space-x-4">
                <a
                  href="/app"
                  className="sacred-button sacred-button-outline"
                >
                  Launch App
                </a>
              </div>
            </nav>
          </div>
        </header>

        {/* Main Content */}
        <main className="flex-1">
          {children}
        </main>

        {/* Footer */}
        <footer className="border-t border-sacred-gray-200 mt-auto">
          <div className="sacred-container py-12">
            <div className="grid grid-cols-1 md:grid-cols-4 gap-8">
              <div>
                <h3 className="font-mono text-sm uppercase tracking-wider mb-4">Product</h3>
                <ul className="space-y-2">
                  <li>
                    <a href="/reserves" className="text-sm text-sacred-gray-600 hover:text-sacred-black transition-colors">
                      Reserve Dashboard
                    </a>
                  </li>
                  <li>
                    <a href="/app" className="text-sm text-sacred-gray-600 hover:text-sacred-black transition-colors">
                      Institutional Portal
                    </a>
                  </li>
                  <li>
                    <a href="/developers" className="text-sm text-sacred-gray-600 hover:text-sacred-black transition-colors">
                      Developer Portal
                    </a>
                  </li>
                </ul>
              </div>
              
              <div>
                <h3 className="font-mono text-sm uppercase tracking-wider mb-4">Resources</h3>
                <ul className="space-y-2">
                  <li>
                    <a href="/docs" className="text-sm text-sacred-gray-600 hover:text-sacred-black transition-colors">
                      Documentation
                    </a>
                  </li>
                  <li>
                    <a href="/docs/api" className="text-sm text-sacred-gray-600 hover:text-sacred-black transition-colors">
                      API Reference
                    </a>
                  </li>
                  <li>
                    <a href="https://github.com/mjohnson518/meridian" className="text-sm text-sacred-gray-600 hover:text-sacred-black transition-colors">
                      GitHub
                    </a>
                  </li>
                </ul>
              </div>
              
              <div>
                <h3 className="font-mono text-sm uppercase tracking-wider mb-4">Contracts</h3>
                <ul className="space-y-2 font-mono text-xs">
                  <li>
                    <span className="text-sacred-gray-600">Factory:</span>
                    <br />
                    <a 
                      href="https://sepolia.etherscan.io/address/0xbe35619896F963dD0Eac90A93A135c53547499C9"
                      className="break-all hover:opacity-70"
                    >
                      0xbe3561...7499C9
                    </a>
                  </li>
                  <li>
                    <span className="text-sacred-gray-600">EUR:</span>
                    <br />
                    <a 
                      href="https://sepolia.etherscan.io/address/0xDcd19e3b07AB23F6771aDda3ab76d7e6823B5D2f"
                      className="break-all hover:opacity-70"
                    >
                      0xDcd19e...23B5D2f
                    </a>
                  </li>
                </ul>
              </div>
              
              <div>
                <h3 className="font-mono text-sm uppercase tracking-wider mb-4">Network</h3>
                <p className="text-sm text-sacred-gray-600 mb-2">
                  Currently on Sepolia Testnet
                </p>
                <div className="flex items-center space-x-1">
                  <div className="w-2 h-2 rounded-full bg-emerald-600 animate-pulse" />
                  <span className="text-xs font-mono uppercase">Active</span>
                </div>
              </div>
            </div>
            
            <div className="mt-12 pt-8 border-t border-sacred-gray-100">
              <p className="text-xs text-sacred-gray-500 text-center font-mono">
                © 2024 Meridian Finance. All rights reserved.
              </p>
            </div>
          </div>
        </footer>
      </body>
    </html>
  );
}
