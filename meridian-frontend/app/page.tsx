'use client';

import { SacredButton } from '@/components/sacred/Button';
import { SacredCard } from '@/components/sacred/Card';
import { SacredGrid } from '@/components/sacred/Grid';
import { Heading } from '@/components/sacred/Typography';

export default function HomePage() {
  return (
    <div className="min-h-screen bg-background dark:bg-background-dark text-foreground dark:text-foreground-dark transition-colors duration-300">
      {/* Hero Section */}
      <section className="relative min-h-[90vh] flex items-center justify-center overflow-hidden">
        {/* Background Gradient */}
        <div className="absolute inset-0 bg-gradient-to-br from-indigo-500/20 via-purple-500/20 to-pink-500/20 dark:from-indigo-900/40 dark:via-purple-900/40 dark:to-pink-900/40 z-0" />

        {/* Animated Background Shapes */}
        <div className="absolute top-20 left-20 w-72 h-72 bg-blue-400/30 rounded-full mix-blend-multiply filter blur-xl opacity-70 animate-blob" />
        <div className="absolute top-20 right-20 w-72 h-72 bg-purple-400/30 rounded-full mix-blend-multiply filter blur-xl opacity-70 animate-blob animation-delay-2000" />
        <div className="absolute -bottom-8 left-20 w-72 h-72 bg-pink-400/30 rounded-full mix-blend-multiply filter blur-xl opacity-70 animate-blob animation-delay-4000" />

        <div className="relative z-10 max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 text-center">
          <div className="inline-block mb-8 animate-fade-in">
            <span className="px-4 py-2 rounded-full bg-white/10 backdrop-blur-md border border-white/20 text-sm font-medium text-foreground dark:text-white shadow-lg">
              ✨ Live on Sepolia Testnet
            </span>
          </div>

          <h1 className="text-5xl md:text-7xl font-extrabold tracking-tight mb-6 animate-slide-up">
            Banking Infrastructure<br />
            <span className="bg-clip-text text-transparent bg-gradient-to-r from-amber-400 to-orange-500">
              for the Future
            </span>
          </h1>

          <p className="text-xl md:text-2xl text-muted dark:text-gray-300 max-w-3xl mx-auto mb-12 animate-slide-up animation-delay-200">
            Multi-currency stablecoins backed by sovereign bonds. Built for global finance with enterprise-grade compliance.
          </p>

          <div className="flex flex-col sm:flex-row gap-4 justify-center animate-slide-up animation-delay-400">
            <a href="/reserves" className="px-8 py-4 text-lg font-semibold rounded-lg bg-white text-purple-700 shadow-xl hover:shadow-2xl hover:-translate-y-1 transition-all duration-300">
              View Live Demo →
            </a>
            <a href="/docs" className="px-8 py-4 text-lg font-semibold rounded-lg border-2 border-foreground/20 hover:border-foreground/40 backdrop-blur-sm transition-all duration-300">
              Documentation
            </a>
          </div>
        </div>
      </section>

      {/* Stats Section */}
      <section className="py-20 bg-gray-50 dark:bg-zinc-900/50">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-8">
            {[
              { label: 'Total Value Locked', value: '$10,042,250', color: 'text-emerald-600 dark:text-emerald-400' },
              { label: 'Reserve Ratio', value: '100.42%', color: 'text-blue-600 dark:text-blue-400' },
              { label: 'Active Currencies', value: '4', color: 'text-purple-600 dark:text-purple-400' },
              { label: 'Transactions Today', value: '1,247', color: 'text-amber-600 dark:text-amber-400' }
            ].map((stat, i) => (
              <div key={i} className="bg-white dark:bg-zinc-800 p-8 rounded-2xl shadow-lg hover:shadow-xl transition-all duration-300 hover:-translate-y-1 border border-gray-100 dark:border-zinc-700">
                <p className="text-sm font-medium text-muted dark:text-gray-400 mb-2 uppercase tracking-wider">
                  {stat.label}
                </p>
                <p className={`text-3xl font-bold ${stat.color} font-mono`}>
                  {stat.value}
                </p>
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* Features Section */}
      <section className="py-24 relative overflow-hidden">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 relative z-10">
          <div className="text-center mb-16">
            <h2 className="text-4xl font-bold mb-4">
              Enterprise-Grade Infrastructure
            </h2>
            <p className="text-xl text-muted dark:text-gray-400">
              Built for the next generation of financial applications
            </p>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-3 gap-8">
            {[
              { title: 'Multi-Currency Support', desc: 'Deploy stablecoins for EUR, GBP, JPY, and emerging markets', icon: '🌍' },
              { title: 'x402 Agent Payments', desc: 'Built for the agentic economy with native AI payment protocols', icon: '🤖' },
              { title: 'Regulatory Compliant', desc: 'GENIUS Act, MiCA, and AML/KYC requirements built-in', icon: '✅' }
            ].map((feature, i) => (
              <div key={i} className="group p-8 rounded-2xl bg-white dark:bg-zinc-800 border border-gray-200 dark:border-zinc-700 hover:border-purple-500 dark:hover:border-purple-500 transition-all duration-300 hover:shadow-2xl hover:shadow-purple-500/10">
                <div className="text-5xl mb-6 transform group-hover:scale-110 transition-transform duration-300">
                  {feature.icon}
                </div>
                <h3 className="text-2xl font-bold mb-4 group-hover:text-purple-600 dark:group-hover:text-purple-400 transition-colors">
                  {feature.title}
                </h3>
                <p className="text-muted dark:text-gray-400 leading-relaxed">
                  {feature.desc}
                </p>
              </div>
            ))}
          </div>
        </div>
      </section>
    </div>
  );
}
