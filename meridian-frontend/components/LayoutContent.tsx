'use client';

import { ThemeToggle } from './ThemeToggle';

export function LayoutContent({ children }: { children: React.ReactNode }) {
  return (
    <>
      {/* Header */}
      <header className="border-b border-gray-200 dark:border-gray-800 bg-white/80 dark:bg-[#0A0A0B]/80 backdrop-blur-md sticky top-0 z-50 transition-colors">
        <div className="max-w-[1200px] mx-auto px-8">
          <nav className="flex items-center justify-between h-16">
            <div className="flex items-center space-x-12">
              <a href="/" className="text-lg font-medium tracking-tight no-underline hover:opacity-70 transition-opacity text-gray-900 dark:text-gray-100">
                MERIDIAN
              </a>
              <div className="hidden md:flex items-center space-x-8">
                <a 
                  href="/reserves" 
                  className="text-sm font-medium text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-100 transition-colors no-underline"
                >
                  Reserves
                </a>
                <a 
                  href="/portal/login" 
                  className="text-sm font-medium text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-100 transition-colors no-underline"
                >
                  Portal
                </a>
                <a 
                  href="/developers" 
                  className="text-sm font-medium text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-100 transition-colors no-underline"
                >
                  Developers
                </a>
                <a 
                  href="/docs" 
                  className="text-sm font-medium text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-100 transition-colors no-underline"
                >
                  Docs
                </a>
              </div>
            </div>
            
            <div className="flex items-center space-x-4">
              <ThemeToggle />
              <a
                href="/portal/login"
                className="px-4 py-2 text-sm font-medium bg-gray-900 dark:bg-white text-white dark:text-gray-900 hover:bg-gray-800 dark:hover:bg-gray-100 rounded transition-all duration-200"
              >
                Launch Portal
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
      <footer className="border-t border-gray-200 dark:border-gray-800 mt-auto bg-gray-50 dark:bg-[#141416]">
        <div className="max-w-[1200px] mx-auto px-8 py-16">
          <div className="grid grid-cols-1 md:grid-cols-4 gap-12">
            <div>
              <h3 className="text-sm font-medium tracking-wider mb-4 text-gray-900 dark:text-gray-100">Product</h3>
              <ul className="space-y-2">
                <li>
                  <a href="/reserves" className="text-sm text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-100 transition-colors">
                    Reserve Dashboard
                  </a>
                </li>
                <li>
                  <a href="/app" className="text-sm text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-100 transition-colors">
                    Institutional Portal
                  </a>
                </li>
                <li>
                  <a href="/developers" className="text-sm text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-100 transition-colors">
                    Developer Portal
                  </a>
                </li>
              </ul>
            </div>
            
            <div>
              <h3 className="text-sm font-medium tracking-wider mb-4 text-gray-900 dark:text-gray-100">Resources</h3>
              <ul className="space-y-2">
                <li>
                  <a href="/docs" className="text-sm text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-100 transition-colors">
                    Documentation
                  </a>
                </li>
                <li>
                  <a href="/docs/api" className="text-sm text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-100 transition-colors">
                    API Reference
                  </a>
                </li>
                <li>
                  <a href="https://github.com/mjohnson518/meridian" className="text-sm text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-100 transition-colors">
                    GitHub
                  </a>
                </li>
              </ul>
            </div>
            
            <div>
              <h3 className="text-sm font-medium tracking-wider mb-4 text-gray-900 dark:text-gray-100">Contracts</h3>
              <ul className="space-y-2 font-mono text-xs">
                <li>
                  <span className="text-gray-600 dark:text-gray-400">Factory:</span>
                  <br />
                  <a 
                    href="https://sepolia.etherscan.io/address/0xbe35619896F963dD0Eac90A93A135c53547499C9"
                    className="break-all hover:opacity-70 text-gray-900 dark:text-gray-100"
                  >
                    0xbe3561...7499C9
                  </a>
                </li>
                <li>
                  <span className="text-gray-600 dark:text-gray-400">EUR:</span>
                  <br />
                  <a 
                    href="https://sepolia.etherscan.io/address/0xDcd19e3b07AB23F6771aDda3ab76d7e6823B5D2f"
                    className="break-all hover:opacity-70 text-gray-900 dark:text-gray-100"
                  >
                    0xDcd19e...23B5D2f
                  </a>
                </li>
              </ul>
            </div>
            
            <div>
              <h3 className="text-sm font-medium tracking-wider mb-4 text-gray-900 dark:text-gray-100">Network</h3>
              <p className="text-sm text-gray-600 dark:text-gray-400 mb-2">
                Currently on Sepolia Testnet
              </p>
              <div className="flex items-center space-x-1">
                <div className="w-2 h-2 rounded-full bg-emerald-600 animate-pulse" />
                <span className="text-xs font-mono uppercase text-gray-600 dark:text-gray-400">Active</span>
              </div>
            </div>
          </div>
          
          <div className="mt-12 pt-8 border-t border-gray-200 dark:border-gray-800">
            <p className="text-xs text-gray-500 dark:text-gray-400 text-center font-mono">
              © 2025 Meridian Finance. All rights reserved.
            </p>
          </div>
        </div>
      </footer>
    </>
  );
}

