'use client';

import { useEffect, useState } from 'react';
import CountUp from 'react-countup';
import { motion } from 'framer-motion';
import { Shield, TrendingUp, Globe, Zap, Lock, BarChart3, ArrowRight } from 'lucide-react';
import { Hero, Card, Button, Badge, MetricCard, AnimatedSection } from '@/components/ui';

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
    <div className="min-h-screen" style={{ backgroundColor: '#FFFFFF', color: '#000000' }} data-theme="light">
      {/* TEST COMPONENT - Using inline styles to bypass Tailwind */}
      <div 
        style={{ 
          backgroundColor: '#10B981', 
          color: '#FFFFFF', 
          padding: '2rem', 
          fontSize: '1.5rem', 
          fontWeight: 'bold', 
          textAlign: 'center',
          borderBottom: '4px solid #0070F3'
        }}
      >
        🧪 TEST: This should be GREEN with white text and blue bottom border (using inline styles)
      </div>
      
      {/* Hero Section */}
      <Hero
        badge={
          <Badge variant="success" size="md">
            <div className="w-1.5 h-1.5 rounded-full bg-emerald-500 animate-pulse" />
            <span>Live on Sepolia Testnet</span>
          </Badge>
        }
        headline="Banking Infrastructure"
        subheadline="for the Agentic Economy"
        description="Multi-currency stablecoins backed by sovereign bonds. Built for global finance with enterprise-grade compliance."
        ctas={{
          primary: (
            <a href="/reserves">
              <Button variant="primary" size="lg">
                View Live Demo
                <ArrowRight className="ml-2 w-4 h-4" />
              </Button>
            </a>
          ),
          secondary: (
            <a href="/docs">
              <Button variant="secondary" size="lg">
                Read Docs
              </Button>
            </a>
          ),
        }}
        trustIndicators={[
          {
            icon: <Shield className="w-4 h-4 text-emerald-500" />,
            text: '100% Backed',
          },
          {
            icon: <Lock className="w-4 h-4 text-blue-500" />,
            text: 'Fully Compliant',
          },
          {
            icon: <BarChart3 className="w-4 h-4 text-emerald-500" />,
            text: 'Real-time Attestation',
          },
        ]}
      />

      {/* Live Stats Section */}
      <section className="border-t border-gray-200 dark:border-gray-800 bg-white dark:bg-black">
        <div className="max-w-7xl mx-auto px-8 py-24">
          <div className="grid grid-cols-2 md:grid-cols-4 gap-8">
            {stats.map((stat, index) => (
              <motion.div
                key={stat.label}
                initial={{ opacity: 0, y: 20 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{ duration: 0.5, delay: index * 0.1 }}
              >
                <MetricCard
                  label={stat.label}
                  value={
                    <>
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
                    </>
                  }
                />
              </motion.div>
            ))}
          </div>
        </div>
      </section>

      {/* Features Section */}
      <section className="bg-white dark:bg-black border-t border-gray-200 dark:border-gray-800">
        <div className="max-w-7xl mx-auto px-8 py-24">
          <AnimatedSection className="text-center mb-16">
            <h2 className="text-5xl md:text-6xl font-bold mb-6 text-black dark:text-white tracking-tight" style={{ letterSpacing: '-0.02em' }}>
              Built for Global Finance
            </h2>
            <p className="text-xl text-gray-600 dark:text-gray-400 max-w-2xl mx-auto">
              Enterprise-grade infrastructure with full regulatory compliance
            </p>
          </AnimatedSection>

          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
            {features.map((feature, index) => {
              const Icon = feature.icon;
              return (
                <AnimatedSection key={feature.title} delay={index * 0.08}>
                  <Card hover className="h-full group">
                    <div className="mb-6">
                      <div className="w-12 h-12 rounded-lg bg-emerald-500/10 dark:bg-emerald-500/20 flex items-center justify-center group-hover:bg-emerald-500/20 dark:group-hover:bg-emerald-500/30 transition-colors">
                        <Icon className="w-6 h-6 text-emerald-600 dark:text-emerald-400" />
                      </div>
                    </div>
                    <h3 className="text-xl font-bold mb-3 text-black dark:text-white">
                      {feature.title}
                    </h3>
                    <p className="text-base text-gray-600 dark:text-gray-400 leading-relaxed">
                      {feature.description}
                    </p>
                  </Card>
                </AnimatedSection>
              );
            })}
          </div>
        </div>
      </section>

      {/* Code Example Section */}
      <section className="border-t border-gray-200 dark:border-gray-800 bg-white dark:bg-black">
        <div className="max-w-7xl mx-auto px-8 py-24">
          <div className="max-w-4xl mx-auto">
            <AnimatedSection className="text-center mb-12">
              <h2 className="text-5xl md:text-6xl font-bold mb-6 text-black dark:text-white tracking-tight" style={{ letterSpacing: '-0.02em' }}>
                Simple Integration
              </h2>
              <p className="text-xl text-gray-600 dark:text-gray-400">
                Start minting stablecoins with just a few lines of code
              </p>
            </AnimatedSection>
            
            <AnimatedSection delay={0.2}>
              <Card className="bg-black dark:bg-gray-900 text-white border-gray-800 dark:border-gray-700" padding="lg">
                <pre className="font-mono text-sm overflow-x-auto">
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
              </Card>
            </AnimatedSection>
          </div>
        </div>
      </section>

      {/* CTA Section */}
      <section className="border-t border-gray-200 dark:border-gray-800 bg-white dark:bg-black">
        <div className="max-w-7xl mx-auto px-8 py-24">
          <AnimatedSection className="max-w-3xl mx-auto text-center">
            <h2 className="text-5xl md:text-6xl font-bold mb-6 text-black dark:text-white tracking-tight" style={{ letterSpacing: '-0.02em' }}>
              Ready to Build?
            </h2>
            <p className="text-xl text-gray-600 dark:text-gray-400 mb-12 max-w-2xl mx-auto">
              Launch your own multi-currency stablecoin in minutes, not months.
            </p>
            <div className="flex flex-col sm:flex-row gap-4 justify-center">
              <a href="/portal/login">
                <Button variant="primary" size="lg">
                  Launch Portal
                  <ArrowRight className="ml-2 w-4 h-4" />
                </Button>
              </a>
              <a href="https://github.com/mjohnson518/meridian" target="_blank" rel="noopener">
                <Button variant="secondary" size="lg">
                  View on GitHub
                </Button>
              </a>
            </div>
          </AnimatedSection>
        </div>
      </section>
    </div>
  );
}
