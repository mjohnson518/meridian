'use client';

import { useEffect, useState } from 'react';
import CountUp from 'react-countup';
import { motion } from 'framer-motion';
import { Shield, TrendingUp, Globe, Zap, Lock, BarChart3, ArrowRight } from 'lucide-react';
import { Button } from '@/components/ui';

export default function HomePage() {
  const [mounted, setMounted] = useState(false);

  useEffect(() => {
    setMounted(true);
  }, []);

  const stats = [
    { label: 'Total Value Locked', value: 10042250, prefix: '$', decimals: 0, change: '+12.5%' },
    { label: 'Reserve Ratio', value: 100.42, suffix: '%', decimals: 2, change: 'Healthy' },
    { label: 'Currencies Live', value: 4, decimals: 0, change: 'EUR, GBP, JPY, MXN' },
    { label: 'Transactions Today', value: 247, decimals: 0, change: 'Last 24h' },
  ];

  const features = [
    {
      icon: Globe,
      title: 'Multi-Currency Support',
      description: 'Deploy stablecoins for any currency - EUR, GBP, JPY, and emerging markets.',
    },
    {
      icon: Shield,
      title: 'Sovereign Bond Backing',
      description: '100% backed by government bonds with real-time attestation and transparency.',
    },
    {
      icon: BarChart3,
      title: 'Chainlink Oracle Integration',
      description: 'Real-time FX rates from decentralized oracle networks for accurate pricing.',
    },
    {
      icon: Zap,
      title: 'x402 Agent Payments',
      description: 'Built for the agentic economy with native AI agent payment protocols.',
    },
    {
      icon: Lock,
      title: 'Institutional Grade',
      description: 'Bank-level compliance with GENIUS Act, MiCA, and AML/KYC requirements.',
    },
    {
      icon: TrendingUp,
      title: 'Open Source',
      description: 'Fully auditable smart contracts and transparent reserve management.',
    },
  ];

  return (
    <div className="min-h-screen bg-white dark:bg-black text-black dark:text-white">
      {/* Hero Section - Linear/Stripe Inspired */}
      <section className="relative min-h-[85vh] flex items-center justify-center overflow-hidden">
        {/* Gradient Background */}
        <div className="absolute inset-0 bg-gradient-to-br from-emerald-50 via-white to-blue-50 dark:from-gray-950 dark:via-black dark:to-gray-950" />
        
        {/* Grid Pattern Overlay */}
        <div 
          className="absolute inset-0 opacity-30 dark:opacity-20"
          style={{
            backgroundImage: `url("data:image/svg+xml,%3Csvg width='60' height='60' xmlns='http://www.w3.org/2000/svg'%3E%3Cdefs%3E%3Cpattern id='grid' width='60' height='60' patternUnits='userSpaceOnUse'%3E%3Cpath d='M 60 0 L 0 0 0 60' fill='none' stroke='gray' stroke-width='0.5'/%3E%3C/pattern%3E%3C/defs%3E%3Crect width='100%25' height='100%25' fill='url(%23grid)'/%3E%3C/svg%3E")`,
          }}
        />

        <div className="relative z-10 max-w-7xl mx-auto px-6 sm:px-8 md:px-12 lg:px-16 xl:px-24 text-center">
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.6 }}
          >
            {/* Badge */}
            <div className="inline-flex items-center gap-2 px-4 py-2 bg-emerald-500/10 dark:bg-emerald-500/20 text-emerald-600 dark:text-emerald-400 rounded-full text-sm font-medium mb-8">
              <span className="w-2 h-2 bg-emerald-500 rounded-full animate-pulse" />
              Live on Sepolia Testnet
            </div>

            {/* Hero Headline - MUCH LARGER */}
            <h1 className="text-7xl sm:text-8xl md:text-9xl font-bold tracking-tight mb-8 leading-[1.1]">
              <span className="block bg-gradient-to-r from-black to-gray-600 dark:from-white dark:to-gray-400 bg-clip-text text-transparent">
                Banking
              </span>
              <span className="block bg-gradient-to-r from-emerald-600 to-blue-600 bg-clip-text text-transparent">
                Infrastructure
              </span>
            </h1>

            {/* Subheadline */}
            <p className="text-xl sm:text-2xl md:text-3xl text-gray-600 dark:text-gray-400 max-w-4xl mx-auto mb-12 font-light">
              Multi-currency stablecoins backed by sovereign bonds.<br />
              Built for global finance with enterprise-grade compliance.
            </p>

            {/* CTAs */}
            <div className="flex flex-col sm:flex-row gap-4 justify-center mb-16">
              <a href="/reserves">
                <Button variant="primary" size="lg">
                  View Live Demo
                  <ArrowRight className="ml-2 w-5 h-5" />
                </Button>
              </a>
              <a href="/docs">
                <Button variant="secondary" size="lg">
                  Read Documentation
                </Button>
              </a>
            </div>

            {/* Trust Indicators */}
            <div className="flex flex-wrap items-center justify-center gap-8">
              <div className="flex items-center space-x-2 text-gray-600 dark:text-gray-400">
                <Shield className="w-5 h-5 text-emerald-500" />
                <span className="font-medium">100% Backed</span>
              </div>
              <div className="flex items-center space-x-2 text-gray-600 dark:text-gray-400">
                <Lock className="w-5 h-5 text-blue-500" />
                <span className="font-medium">Fully Compliant</span>
              </div>
              <div className="flex items-center space-x-2 text-gray-600 dark:text-gray-400">
                <BarChart3 className="w-5 h-5 text-emerald-500" />
                <span className="font-medium">Real-time Attestation</span>
              </div>
            </div>
          </motion.div>
        </div>
      </section>

      {/* Stats Section - Professional Cards */}
      <section className="py-20 sm:py-24 lg:py-32 bg-gray-50 dark:bg-gray-950 border-t border-gray-200 dark:border-gray-800">
        <div className="max-w-7xl mx-auto px-6 sm:px-8 md:px-12 lg:px-16 xl:px-24">
          <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-6 sm:gap-8">
            {stats.map((stat, index) => (
              <motion.div
                key={stat.label}
                initial={{ opacity: 0, y: 20 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{ duration: 0.5, delay: index * 0.1 }}
                className="bg-white dark:bg-gray-900 p-6 sm:p-8 rounded-2xl border border-gray-200 dark:border-gray-800 hover:border-emerald-500 hover:shadow-xl transition-all"
              >
                <p className="text-sm text-gray-500 dark:text-gray-400 mb-3 font-medium uppercase tracking-wider">
                  {stat.label}
                </p>
                <p className="text-3xl sm:text-4xl font-bold mb-2 text-black dark:text-white">
                  {stat.prefix}
                  {mounted ? (
                    <CountUp
                      end={stat.value}
                      decimals={stat.decimals}
                      duration={2.5}
                      separator=","
                    />
                  ) : (
                    stat.value.toLocaleString()
                  )}
                  {stat.suffix}
                </p>
                <p className="text-sm text-emerald-600 dark:text-emerald-400 font-medium">
                  {stat.change}
                </p>
              </motion.div>
            ))}
          </div>
        </div>
      </section>

      {/* Features Section */}
      <section className="py-20 sm:py-24 lg:py-32 bg-white dark:bg-black">
        <div className="max-w-7xl mx-auto px-6 sm:px-8 md:px-12 lg:px-16 xl:px-24">
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.6 }}
            className="text-center mb-16 sm:mb-20"
          >
            <h2 className="text-5xl sm:text-6xl md:text-7xl font-bold mb-6 text-black dark:text-white tracking-tight" style={{ letterSpacing: '-0.02em' }}>
              Built for Global Finance
            </h2>
            <p className="text-xl sm:text-2xl text-gray-600 dark:text-gray-400 max-w-3xl mx-auto">
              Enterprise-grade infrastructure with full regulatory compliance
            </p>
          </motion.div>

          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6 sm:gap-8">
            {features.map((feature, index) => {
              const Icon = feature.icon;
              return (
                <motion.div
                  key={feature.title}
                  initial={{ opacity: 0, y: 20 }}
                  animate={{ opacity: 1, y: 0 }}
                  transition={{ duration: 0.5, delay: index * 0.08 }}
                  className="group p-8 sm:p-10 rounded-2xl border border-gray-200 dark:border-gray-800 hover:border-emerald-500 hover:shadow-xl transition-all bg-white dark:bg-gray-900"
                >
                  <div className="w-14 h-14 bg-emerald-500/10 dark:bg-emerald-500/20 rounded-xl flex items-center justify-center mb-6 group-hover:scale-110 group-hover:bg-emerald-500/20 dark:group-hover:bg-emerald-500/30 transition-all">
                    <Icon className="w-7 h-7 text-emerald-600 dark:text-emerald-400" />
                  </div>
                  <h3 className="text-xl sm:text-2xl font-bold mb-3 text-black dark:text-white">
                    {feature.title}
                  </h3>
                  <p className="text-base text-gray-600 dark:text-gray-400 leading-relaxed">
                    {feature.description}
                  </p>
                </motion.div>
              );
            })}
          </div>
        </div>
      </section>

      {/* Code Example Section */}
      <section className="py-20 sm:py-24 lg:py-32 bg-gray-50 dark:bg-gray-950 border-t border-gray-200 dark:border-gray-800">
        <div className="max-w-7xl mx-auto px-6 sm:px-8 md:px-12 lg:px-16 xl:px-24">
          <div className="max-w-5xl mx-auto">
            <motion.div
              initial={{ opacity: 0, y: 20 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ duration: 0.5 }}
              className="text-center mb-12 sm:mb-16"
            >
              <h2 className="text-5xl sm:text-6xl md:text-7xl font-bold mb-6 text-black dark:text-white tracking-tight" style={{ letterSpacing: '-0.02em' }}>
                Simple Integration
              </h2>
              <p className="text-xl sm:text-2xl text-gray-600 dark:text-gray-400">
                Start minting stablecoins with just a few lines of code
              </p>
            </motion.div>
            
            <motion.div
              initial={{ opacity: 0, y: 20 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ duration: 0.5, delay: 0.2 }}
              className="bg-black dark:bg-gray-900 rounded-2xl border border-gray-800 dark:border-gray-700 p-8 sm:p-10 overflow-hidden"
            >
              <pre className="font-mono text-sm sm:text-base overflow-x-auto">
                <code className="text-emerald-400">{`// Mint EUR stablecoins
const tx = await contract.mint(
  userAddress,
  ethers.parseUnits("1000", 18), // 1000 EUR
  reserveProof
);

// Bond requirement automatically calculated
// Settlement: T+1 (next business day)
console.log(\`Minted: \${amount} EUR\`);`}</code>
              </pre>
            </motion.div>
          </div>
        </div>
      </section>

      {/* CTA Section */}
      <section className="py-20 sm:py-24 lg:py-32 bg-white dark:bg-black border-t border-gray-200 dark:border-gray-800">
        <div className="max-w-7xl mx-auto px-6 sm:px-8 md:px-12 lg:px-16 xl:px-24">
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.5 }}
            className="max-w-4xl mx-auto text-center"
          >
            <h2 className="text-5xl sm:text-6xl md:text-7xl font-bold mb-6 sm:mb-8 text-black dark:text-white tracking-tight" style={{ letterSpacing: '-0.02em' }}>
              Ready to Build?
            </h2>
            <p className="text-xl sm:text-2xl text-gray-600 dark:text-gray-400 mb-12 max-w-3xl mx-auto">
              Launch your own multi-currency stablecoin in minutes, not months.
            </p>
            <div className="flex flex-col sm:flex-row gap-4 justify-center">
              <a href="/portal/login">
                <Button variant="primary" size="lg">
                  Launch Portal
                  <ArrowRight className="ml-2 w-5 h-5" />
                </Button>
              </a>
              <a href="https://github.com/mjohnson518/meridian" target="_blank" rel="noopener">
                <Button variant="secondary" size="lg">
                  View on GitHub
                </Button>
              </a>
            </div>
          </motion.div>
        </div>
      </section>
    </div>
  );
}
