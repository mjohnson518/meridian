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
    <div className="min-h-screen">
      {/* Hero Section with Animated Gradient */}
      <section className="relative overflow-hidden">
        {/* Animated Gradient Background */}
        <div className="absolute inset-0 bg-gradient-to-br from-blue-50 via-purple-50 to-pink-50 dark:from-blue-950/20 dark:via-purple-950/20 dark:to-pink-950/20 animate-gradient bg-gradient-animate opacity-60" />
        
        <div className="sacred-container relative z-10 py-24 md:py-32">
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.8 }}
            className="max-w-4xl"
          >
            <div className="inline-flex items-center space-x-2 px-4 py-2 rounded-full bg-brand-emerald/10 dark:bg-brand-emerald/20 border border-brand-emerald/20 dark:border-brand-emerald/30 mb-6">
              <div className="w-2 h-2 rounded-full bg-brand-emerald animate-pulse" />
              <span className="text-sm font-mono text-brand-emerald">Live on Sepolia Testnet</span>
            </div>

            <h1 className="text-5xl md:text-6xl lg:text-7xl font-medium tracking-tight mb-6 text-gray-900 dark:text-gray-100">
              Multi-Currency Stablecoin Infrastructure
            </h1>
            
            <p className="text-xl md:text-2xl text-gray-600 dark:text-gray-400 mb-8 leading-relaxed max-w-3xl">
              Professional-grade platform for launching compliant, multi-currency stablecoins backed by sovereign bonds. 
              Built for institutions and the agentic economy.
            </p>

            <div className="flex flex-col sm:flex-row gap-4">
              <motion.div whileHover={{ scale: 1.02 }} whileTap={{ scale: 0.98 }}>
                <a href="/reserves">
                  <SacredButton variant="primary" size="lg">
                    View Reserves →
                  </SacredButton>
                </a>
              </motion.div>
              <motion.div whileHover={{ scale: 1.02 }} whileTap={{ scale: 0.98 }}>
                <a href="https://github.com/mjohnson518/meridian" target="_blank" rel="noopener">
                  <SacredButton variant="outline" size="lg">
                    Documentation
                  </SacredButton>
                </a>
              </motion.div>
            </div>

            {/* Trust Badges */}
            <div className="mt-12 flex flex-wrap items-center gap-6 text-sm font-mono text-gray-500 dark:text-gray-400">
              <div className="flex items-center space-x-2">
                <Shield className="w-4 h-4" />
                <span>100% Backed</span>
              </div>
              <div className="flex items-center space-x-2">
                <Lock className="w-4 h-4" />
                <span>GENIUS Act Compliant</span>
              </div>
              <div className="flex items-center space-x-2">
                <BarChart3 className="w-4 h-4" />
                <span>Real-time Attestation</span>
              </div>
            </div>
          </motion.div>
        </div>
      </section>

      {/* Live Stats Section */}
      <section className="border-y border-gray-200 dark:border-gray-800 bg-gray-50 dark:bg-gray-900">
        <div className="sacred-container py-12">
          <SacredGrid cols={4} gap={8}>
            {stats.map((stat, index) => (
              <motion.div
                key={stat.label}
                initial={{ opacity: 0, y: 20 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{ duration: 0.5, delay: index * 0.1 }}
                className="text-center"
              >
                <p className="text-xs font-mono uppercase tracking-wider text-gray-600 dark:text-gray-400 mb-2">
                  {stat.label}
                </p>
                <div className="font-mono text-2xl md:text-3xl tabular-nums text-gray-900 dark:text-gray-100">
                  {stat.prefix}
                  {mounted ? (
                    <CountUp
                      end={stat.value}
                      decimals={stat.decimals}
                      duration={2}
                      separator=","
                    />
                  ) : (
                    stat.value.toLocaleString()
                  )}
                  {stat.suffix}
                </div>
              </motion.div>
            ))}
          </SacredGrid>
        </div>
      </section>

      {/* Features Section */}
      <section className="sacred-container py-24">
        <motion.div
          initial={{ opacity: 0 }}
          whileInView={{ opacity: 1 }}
          viewport={{ once: true }}
          transition={{ duration: 0.6 }}
        >
          <Heading level={2} className="text-3xl md:text-4xl mb-4 text-center">
            Built for the New Financial System
          </Heading>
          <p className="text-center text-gray-600 dark:text-gray-400 mb-12 max-w-2xl mx-auto">
            Enterprise-grade infrastructure for multi-currency stablecoins with full regulatory compliance
          </p>
        </motion.div>

        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
          {features.map((feature, index) => {
            const Icon = feature.icon;
            return (
              <motion.div
                key={feature.title}
                initial={{ opacity: 0, y: 20 }}
                whileInView={{ opacity: 1, y: 0 }}
                viewport={{ once: true }}
                transition={{ duration: 0.5, delay: index * 0.1 }}
                whileHover={{ y: -4, transition: { duration: 0.2 } }}
              >
                <SacredCard className="h-full hover:shadow-lg dark:hover:shadow-brand-emerald/5 transition-shadow duration-300">
                  <div className="mb-4">
                    <div className="w-12 h-12 rounded-lg bg-brand-emerald/10 dark:bg-brand-emerald/20 flex items-center justify-center">
                      <Icon className="w-6 h-6 text-brand-emerald" />
                    </div>
                  </div>
                  <Heading level={3} className="text-lg font-medium mb-3">
                    {feature.title}
                  </Heading>
                  <p className="text-sm text-gray-600 dark:text-gray-400 leading-relaxed">
                    {feature.description}
                  </p>
                </SacredCard>
              </motion.div>
            );
          })}
        </div>
      </section>

      {/* Code Example Section */}
      <section className="border-t border-gray-200 dark:border-gray-800 bg-gray-50 dark:bg-gray-900">
        <div className="sacred-container py-24">
          <div className="max-w-3xl mx-auto">
            <motion.div
              initial={{ opacity: 0 }}
              whileInView={{ opacity: 1 }}
              viewport={{ once: true }}
            >
              <Heading level={2} className="text-3xl md:text-4xl mb-8 text-center">
                Simple Integration
              </Heading>
              
              <div className="bg-gray-900 dark:bg-gray-950 text-white p-6 rounded-lg font-mono text-sm overflow-x-auto border border-gray-800 dark:border-gray-700 shadow-2xl">
                <pre className="text-brand-emerald">{`// Mint EUR stablecoins
const tx = await contract.mint(
  userAddress,
  ethers.parseUnits("1000", 18), // 1000 EUR
  reserveProof
);

// Bond requirement automatically calculated
// Settlement: T+1 (next business day)
console.log(\`Minted: \${amount} EUR\`);`}</pre>
              </div>
            </motion.div>
          </div>
        </div>
      </section>

      {/* CTA Section */}
      <section className="sacred-container py-24">
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          className="max-w-2xl mx-auto text-center"
        >
          <Heading level={2} className="text-3xl md:text-4xl mb-6">
            Ready to Build?
          </Heading>
          <p className="text-lg text-gray-600 dark:text-gray-400 mb-8">
            Launch your own multi-currency stablecoin in minutes, not months.
          </p>
          <div className="flex flex-col sm:flex-row gap-4 justify-center">
            <motion.div whileHover={{ scale: 1.05 }} whileTap={{ scale: 0.95 }}>
              <a href="/portal/login">
                <SacredButton variant="primary" size="lg">
                  Launch Portal →
                </SacredButton>
              </a>
            </motion.div>
            <motion.div whileHover={{ scale: 1.05 }} whileTap={{ scale: 0.95 }}>
              <a href="https://github.com/mjohnson518/meridian" target="_blank" rel="noopener">
                <SacredButton variant="outline" size="lg">
                  View on GitHub
                </SacredButton>
              </a>
            </motion.div>
          </div>
        </motion.div>
      </section>
    </div>
  );
}
