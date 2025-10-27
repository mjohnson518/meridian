'use client';

import { SacredCard } from '@/components/sacred/Card';
import { SacredButton } from '@/components/sacred/Button';
import { SacredGrid } from '@/components/sacred/Grid';
import { Heading } from '@/components/sacred/Typography';
import { MonoDisplay } from '@/components/sacred/Typography';

export default function HomePage() {
  const stats = [
    { label: 'Total Value Locked', value: 10042250, format: 'currency' },
    { label: 'Reserve Ratio', value: 100.42, format: 'percentage' },
    { label: 'Currencies Live', value: 1, format: 'number' },
    { label: 'Attestations Today', value: 24, format: 'number' },
  ];

  const features = [
    {
      title: 'Multi-Currency Support',
      description: 'Deploy stablecoins for any currency - EUR, GBP, JPY, and emerging markets.',
    },
    {
      title: 'Sovereign Bond Backing',
      description: '100% backed by government bonds with real-time attestation and transparency.',
    },
    {
      title: 'Chainlink Oracle Integration',
      description: 'Real-time FX rates from decentralized oracle networks for accurate pricing.',
    },
    {
      title: 'x402 Agent Payments',
      description: 'Built for the agentic economy with native AI agent payment protocols.',
    },
    {
      title: 'Institutional Grade',
      description: 'Bank-level compliance with GENIUS Act, MiCA, and AML/KYC requirements.',
    },
    {
      title: 'Open Source',
      description: 'Fully auditable smart contracts and transparent reserve management.',
    },
  ];

  return (
    <div className="min-h-[calc(100vh-4rem)]">
      {/* Hero Section */}
      <section className="sacred-container py-24">
        <div className="max-w-4xl">
          <Heading level={1} className="text-5xl md:text-6xl mb-6 font-medium">
            Multi-Currency Stablecoin Infrastructure
          </Heading>
          <p className="text-xl text-sacred-gray-600 mb-8 leading-relaxed">
            Professional-grade platform for launching compliant, multi-currency stablecoins 
            backed by sovereign bonds. Built for institutions and the agentic economy.
          </p>
          <div className="flex flex-col sm:flex-row gap-4">
            <SacredButton size="lg" onClick={() => window.location.href = '/reserves'}>
              View Reserves →
            </SacredButton>
            <SacredButton size="lg" variant="outline" onClick={() => window.location.href = '/docs'}>
              Documentation
            </SacredButton>
          </div>
        </div>
      </section>

      {/* Stats Section */}
      <section className="border-y border-sacred-gray-200 bg-sacred-gray-100">
        <div className="sacred-container py-12">
          <div className="grid grid-cols-2 md:grid-cols-4 gap-8">
            {stats.map((stat, index) => (
              <div key={index}>
                <p className="text-xs font-mono uppercase tracking-wider text-sacred-gray-600 mb-2">
                  {stat.label}
                </p>
                {stat.format === 'currency' ? (
                  <MonoDisplay
                    value={stat.value}
                    currency="USD"
                    size="2xl"
                  />
                ) : stat.format === 'percentage' ? (
                  <MonoDisplay
                    value={stat.value}
                    precision={2}
                    suffix="%"
                    size="2xl"
                    color={stat.value >= 100 ? 'positive' : 'negative'}
                  />
                ) : (
                  <span className="font-mono text-2xl tabular-nums">
                    {stat.value}
                  </span>
                )}
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* Features Grid */}
      <section className="sacred-container py-24">
        <Heading level={2} className="text-3xl mb-12 text-center">
          Built for the New Financial System
        </Heading>
        <SacredGrid cols={3} gap={6}>
          {features.map((feature, index) => (
            <SacredCard key={index}>
              <Heading level={3} className="text-lg mb-3 font-mono">
                {feature.title}
              </Heading>
              <p className="text-sm text-sacred-gray-600 leading-relaxed">
                {feature.description}
              </p>
            </SacredCard>
          ))}
        </SacredGrid>
      </section>

      {/* Code Example Section */}
      <section className="border-t border-sacred-gray-200 bg-sacred-gray-100">
        <div className="sacred-container py-24">
          <div className="max-w-3xl mx-auto">
            <Heading level={2} className="text-3xl mb-8">
              Simple Integration
            </Heading>
            <div className="bg-sacred-black text-sacred-white p-6 rounded font-mono text-sm overflow-x-auto">
              <pre>{`// Mint EUR stablecoins
const tx = await contract.mint(
  userAddress,
  ethers.parseUnits("1000", 18), // 1000 EUR
  reserveProof
);

// Check reserve ratio
const ratio = await contract.getReserveRatio();
console.log(\`Reserve Ratio: \${ratio / 100}%\`);

// Agent payment via x402
const payment = await x402.initiatePayment({
  amount: "100",
  currency: "EUR",
  recipient: agentWallet
});`}</pre>
            </div>
          </div>
        </div>
      </section>

      {/* CTA Section */}
      <section className="sacred-container py-24">
        <div className="max-w-2xl mx-auto text-center">
          <Heading level={2} className="text-3xl mb-6">
            Ready to Build?
          </Heading>
          <p className="text-lg text-sacred-gray-600 mb-8">
            Launch your own multi-currency stablecoin in minutes, not months.
          </p>
          <div className="flex flex-col sm:flex-row gap-4 justify-center">
            <SacredButton size="lg" onClick={() => window.location.href = '/app'}>
              Launch Application →
            </SacredButton>
            <SacredButton size="lg" variant="outline" onClick={() => window.location.href = 'https://github.com/mjohnson518/meridian'}>
              View on GitHub
            </SacredButton>
          </div>
        </div>
      </section>
    </div>
  );
}
