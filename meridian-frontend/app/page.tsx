'use client';

import { useEffect, useState } from 'react';
import CountUp from 'react-countup';
import { motion } from 'framer-motion';
import { Shield, TrendingUp, Globe, Zap, Lock, BarChart3 } from 'lucide-react';
import { SacredCard } from '@/components/sacred/Card';
import { SacredButton } from '@/components/sacred/Button';
import { SacredGrid } from '@/components/sacred/Grid';
import { Heading } from '@/components/sacred/Typography';

export default function HomePage() {
  const [mounted, setMounted] = useState(false);

  useEffect(() => {
    setMounted(true);
  }, []);

  const stats = [
    { label: 'Total Value Locked', value: 10042250, prefix: '$', decimals: 0 },
    { label: 'Reserve Ratio', value: 100.42, suffix: '%', decimals: 2 },
    { label: 'Currencies Live', value: 4, decimals: 0 },
    { label: 'Transactions Today', value: 247, decimals: 0 },
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
    <div className="min-h-screen bg-[#FAFAFA] dark:bg-[#0A0A0B]">
      {/* Hero Section */}
      <section className="relative overflow-hidden border-b border-gray-200 dark:border-gray-800">
        {/* Subtle Background Pattern */}
        <div className="absolute inset-0 opacity-[0.015] dark:opacity-[0.03]">
          <div className="absolute inset-0" style={{
            backgroundImage: 'radial-gradient(circle at 1px 1px, rgb(0 0 0) 1px, transparent 0)',
            backgroundSize: '40px 40px'
          }} />
        </div>
        
        <div className="max-w-[1200px] mx-auto px-8 relative z-10 py-32 md:py-40">
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.6 }}
            className="max-w-5xl"
          >
            <motion.div 
              initial={{ opacity: 0, y: 10 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ delay: 0.1, duration: 0.5 }}
              className="inline-flex items-center space-x-2 px-3 py-1.5 rounded-full bg-emerald-50 dark:bg-emerald-950/30 border border-emerald-200 dark:border-emerald-900/50 mb-8"
            >
              <div className="w-1.5 h-1.5 rounded-full bg-emerald-500 animate-pulse" />
              <span className="text-xs font-medium text-emerald-700 dark:text-emerald-400">Live on Sepolia Testnet</span>
            </motion.div>

            <motion.h1 
              initial={{ opacity: 0, y: 10 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ delay: 0.2, duration: 0.5 }}
              className="text-6xl md:text-7xl lg:text-8xl font-medium tracking-tight mb-8 text-gray-900 dark:text-gray-100 leading-[1.1]"
            >
              <span className="block">Multi-Currency</span>
              <span className="block bg-gradient-to-r from-gray-900 via-gray-700 to-gray-900 dark:from-gray-100 dark:via-gray-300 dark:to-gray-100 bg-clip-text text-transparent">
                Stablecoin Platform
              </span>
            </motion.h1>
            
            <motion.p 
              initial={{ opacity: 0, y: 10 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ delay: 0.3, duration: 0.5 }}
              className="text-xl md:text-2xl text-gray-600 dark:text-gray-400 mb-12 leading-relaxed max-w-3xl font-light"
            >
              Enterprise infrastructure for compliant, multi-currency stablecoins backed by sovereign bonds.
              Built for the agentic economy.
            </motion.p>

            <motion.div 
              initial={{ opacity: 0, y: 10 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ delay: 0.4, duration: 0.5 }}
              className="flex flex-col sm:flex-row gap-4"
            >
              <motion.div whileHover={{ scale: 1.02 }} whileTap={{ scale: 0.98 }}>
                <a href="/reserves" className="inline-flex items-center justify-center px-6 py-3 text-sm font-medium bg-gray-900 dark:bg-white text-white dark:text-gray-900 hover:bg-gray-800 dark:hover:bg-gray-100 rounded-lg transition-all duration-200">
                  View Reserves
                  <svg className="ml-2 w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 5l7 7-7 7" />
                  </svg>
                </a>
              </motion.div>
              <motion.div whileHover={{ scale: 1.02 }} whileTap={{ scale: 0.98 }}>
                <a href="https://github.com/mjohnson518/meridian" target="_blank" rel="noopener" className="inline-flex items-center justify-center px-6 py-3 text-sm font-medium bg-transparent text-gray-900 dark:text-gray-100 hover:bg-gray-100 dark:hover:bg-gray-800 border border-gray-300 dark:border-gray-700 rounded-lg transition-all duration-200">
                  Documentation
                </a>
              </motion.div>
            </motion.div>

            {/* Trust Indicators */}
            <motion.div 
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              transition={{ delay: 0.6, duration: 0.5 }}
              className="mt-16 flex flex-wrap items-center gap-8"
            >
              <div className="flex items-center space-x-2 text-gray-600 dark:text-gray-400">
                <Shield className="w-4 h-4" />
                <span className="text-sm">100% Backed</span>
              </div>
              <div className="flex items-center space-x-2 text-gray-600 dark:text-gray-400">
                <Lock className="w-4 h-4" />
                <span className="text-sm">Fully Compliant</span>
              </div>
              <div className="flex items-center space-x-2 text-gray-600 dark:text-gray-400">
                <BarChart3 className="w-4 h-4" />
                <span className="text-sm">Real-time Attestation</span>
              </div>
            </motion.div>
          </motion.div>
        </div>
      </section>

      {/* Live Stats Section */}
      <section className="bg-white dark:bg-[#141416] border-b border-gray-200 dark:border-gray-800">
        <div className="max-w-[1200px] mx-auto px-8 py-20">
          <div className="grid grid-cols-2 md:grid-cols-4 gap-12">
            {stats.map((stat, index) => (
              <motion.div
                key={stat.label}
                initial={{ opacity: 0, y: 20 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{ duration: 0.5, delay: index * 0.1 }}
                className="text-center"
              >
                <p className="text-xs font-medium uppercase tracking-wider text-gray-500 dark:text-gray-400 mb-3">
                  {stat.label}
                </p>
                <div className="font-mono text-3xl md:text-4xl tabular-nums font-medium text-gray-900 dark:text-gray-100">
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
                </div>
              </motion.div>
            ))}
          </div>
        </div>
      </section>

      {/* Features Section */}
      <section className="max-w-[1200px] mx-auto px-8 py-32">
        <motion.div
          initial={{ opacity: 0 }}
          whileInView={{ opacity: 1 }}
          viewport={{ once: true }}
          transition={{ duration: 0.6 }}
          className="text-center mb-20"
        >
          <h2 className="text-4xl md:text-5xl font-medium mb-6 text-gray-900 dark:text-gray-100 tracking-tight">
            Built for Global Finance
          </h2>
          <p className="text-lg text-gray-600 dark:text-gray-400 max-w-2xl mx-auto font-light">
            Enterprise-grade infrastructure with full regulatory compliance
          </p>
        </motion.div>

        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-8">
          {features.map((feature, index) => {
            const Icon = feature.icon;
            return (
              <motion.div
                key={feature.title}
                initial={{ opacity: 0, y: 20 }}
                whileInView={{ opacity: 1, y: 0 }}
                viewport={{ once: true }}
                transition={{ duration: 0.5, delay: index * 0.08 }}
              >
                <motion.div 
                  className="h-full p-8 bg-white dark:bg-[#141416] border border-gray-200 dark:border-gray-800 rounded-xl transition-all duration-300"
                  whileHover={{ y: -4, boxShadow: '0 20px 40px rgba(0,0,0,0.05)' }}
                >
                  <div className="mb-6">
                    <div className="w-12 h-12 rounded-lg bg-emerald-50 dark:bg-emerald-950/30 flex items-center justify-center">
                      <Icon className="w-6 h-6 text-emerald-600 dark:text-emerald-400" />
                    </div>
                  </div>
                  <h3 className="text-lg font-medium mb-3 text-gray-900 dark:text-gray-100">
                    {feature.title}
                  </h3>
                  <p className="text-sm text-gray-600 dark:text-gray-400 leading-relaxed">
                    {feature.description}
                  </p>
                </motion.div>
              </motion.div>
            );
          })}
        </div>
      </section>

      {/* Code Example Section */}
      <section className="border-t border-gray-200 dark:border-gray-800 bg-white dark:bg-[#141416]">
        <div className="max-w-[1200px] mx-auto px-8 py-32">
          <div className="max-w-4xl mx-auto">
            <motion.div
              initial={{ opacity: 0 }}
              whileInView={{ opacity: 1 }}
              viewport={{ once: true }}
              transition={{ duration: 0.6 }}
              className="text-center mb-16"
            >
              <h2 className="text-4xl md:text-5xl font-medium mb-6 text-gray-900 dark:text-gray-100 tracking-tight">
                Simple Integration
              </h2>
              <p className="text-lg text-gray-600 dark:text-gray-400 font-light">
                Start minting stablecoins with just a few lines of code
              </p>
            </motion.div>
            
            <motion.div
              initial={{ opacity: 0, y: 20 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true }}
              transition={{ delay: 0.2, duration: 0.5 }}
              className="bg-[#0A0A0B] dark:bg-black text-gray-100 p-8 rounded-xl font-mono text-sm overflow-x-auto border border-gray-800 dark:border-gray-900 shadow-2xl"
            >
              <pre className="text-emerald-400">{`// Mint EUR stablecoins
const tx = await contract.mint(
  userAddress,
  ethers.parseUnits("1000", 18), // 1000 EUR
  reserveProof
);

// Bond requirement automatically calculated
// Settlement: T+1 (next business day)
console.log(\`Minted: \${amount} EUR\`);`}</pre>
            </motion.div>
          </div>
        </div>
      </section>

      {/* CTA Section */}
      <section className="border-t border-gray-200 dark:border-gray-800 bg-[#FAFAFA] dark:bg-[#0A0A0B]">
        <div className="max-w-[1200px] mx-auto px-8 py-32">
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.6 }}
            className="max-w-3xl mx-auto text-center"
          >
            <h2 className="text-4xl md:text-5xl font-medium mb-6 text-gray-900 dark:text-gray-100 tracking-tight">
              Ready to Build?
            </h2>
            <p className="text-lg text-gray-600 dark:text-gray-400 mb-12 font-light max-w-2xl mx-auto">
              Launch your own multi-currency stablecoin in minutes, not months.
            </p>
            <div className="flex flex-col sm:flex-row gap-4 justify-center">
              <motion.div whileHover={{ scale: 1.02 }} whileTap={{ scale: 0.98 }}>
                <a href="/portal/login" className="inline-flex items-center justify-center px-6 py-3 text-sm font-medium bg-gray-900 dark:bg-white text-white dark:text-gray-900 hover:bg-gray-800 dark:hover:bg-gray-100 rounded-lg transition-all duration-200">
                  Launch Portal
                  <svg className="ml-2 w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 5l7 7-7 7" />
                  </svg>
                </a>
              </motion.div>
              <motion.div whileHover={{ scale: 1.02 }} whileTap={{ scale: 0.98 }}>
                <a href="https://github.com/mjohnson518/meridian" target="_blank" rel="noopener" className="inline-flex items-center justify-center px-6 py-3 text-sm font-medium bg-transparent text-gray-900 dark:text-gray-100 hover:bg-gray-100 dark:hover:bg-gray-800 border border-gray-300 dark:border-gray-700 rounded-lg transition-all duration-200">
                  View on GitHub
                </a>
              </motion.div>
            </div>
          </motion.div>
        </div>
      </section>
    </div>
  );
}
