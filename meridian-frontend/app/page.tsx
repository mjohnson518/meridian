'use client';

import { GlassCard } from '@/components/ui/GlassCard';
import { GlowingButton } from '@/components/ui/GlowingButton';
import { GradientText } from '@/components/ui/GradientText';
// import { FloatingElements } from '@/components/ui/FloatingElements'; // Replaced by EnergySphere
import { ArrowRight, Globe, Shield, Zap, TrendingUp, Building2, Code2 } from 'lucide-react';
import dynamic from 'next/dynamic';

const EnergySphere = dynamic(() => import('@/components/ui/EnergySphere').then(mod => mod.EnergySphere), {
  ssr: false,
  loading: () => <div className="w-full h-full bg-transparent" />
});

export default function HomePage() {
  return (
    <div className="min-h-screen bg-[#050608] text-white overflow-x-hidden selection:bg-emerald-500/30 font-sans relative">

      {/* Deep Space Background & Energy Sphere */}
      <div className="fixed inset-0 z-0 pointer-events-none">

        {/* 3D Energy Sphere - Centered but subtle */}
        <div className="absolute inset-0 z-0 opacity-80">
          <EnergySphere />
        </div>

        {/* Main Nebula - Emerald/Teal - Deeper and richer */}
        {/* <div className="absolute top-[-20%] left-[-10%] w-[80%] h-[80%] bg-[radial-gradient(circle_at_center,_var(--tw-gradient-stops))] from-emerald-900/40 via-[#050608] to-transparent blur-[120px] animate-pulse-slow" /> */}

        {/* Secondary Nebula - Blue/Purple (Keep for depth) */}
        <div className="absolute bottom-[-20%] right-[-10%] w-[80%] h-[80%] bg-[radial-gradient(circle_at_center,_var(--tw-gradient-stops))] from-blue-900/20 via-[#050608] to-transparent blur-[120px] animation-delay-2000 opacity-50" />

        {/* Grid Pattern Overlay */}
        <div className="absolute inset-0 bg-[linear-gradient(to_right,#80808012_1px,transparent_1px),linear-gradient(to_bottom,#80808012_1px,transparent_1px)] bg-[size:24px_24px] [mask-image:radial-gradient(ellipse_60%_50%_at_50%_0%,#000_70%,transparent_100%)] z-10 mix-blend-overlay" />
      </div>

      {/* Navigation */}
      <nav className="relative z-50 border-b border-white/5 bg-[#050608]/50 backdrop-blur-xl">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 h-20 flex items-center justify-between">
          <div className="flex items-center gap-3">
            <div className="w-8 h-8 rounded-lg bg-gradient-to-br from-emerald-400 to-teal-600 flex items-center justify-center shadow-lg shadow-emerald-500/20">
              <Globe className="w-5 h-5 text-white" />
            </div>
            <span className="text-xl font-heading font-bold tracking-tight text-white">Meridian</span>
          </div>
          <div className="hidden md:flex items-center gap-8">
            <a href="#features" className="text-sm font-medium text-gray-400 hover:text-white transition-colors">Features</a>
            <a href="#infrastructure" className="text-sm font-medium text-gray-400 hover:text-white transition-colors">Infrastructure</a>
            <a href="#developers" className="text-sm font-medium text-gray-400 hover:text-white transition-colors">Developers</a>
          </div>
          <div className="flex items-center gap-4">
            <a href="/portal/login" className="text-sm font-medium text-gray-400 hover:text-white transition-colors">Login</a>
            <GlowingButton size="sm" variant="primary" href="/portal/onboarding">Start Building</GlowingButton>
          </div>
        </div>
      </nav>

      {/* Hero Section */}
      <section className="relative z-10 pt-24 pb-32 lg:pt-40 lg:pb-56">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 text-center relative">

          <div className="inline-flex items-center gap-2 px-4 py-2 rounded-full bg-white/5 border border-white/10 mb-8 animate-fade-in backdrop-blur-md shadow-lg hover:bg-white/10 transition-colors cursor-default">
            <span className="w-2 h-2 rounded-full bg-emerald-400 animate-pulse shadow-[0_0_10px_rgba(52,211,153,0.5)]" />
            <span className="text-sm font-medium text-emerald-100">Live on Sepolia Testnet</span>
          </div>

          <h1 className="text-5xl md:text-7xl lg:text-8xl font-heading font-extrabold tracking-tight mb-8 leading-[1.1] animate-slide-up">
            Turnkey Infrastructure for <br />
            <span className="bg-clip-text text-transparent bg-gradient-to-r from-white via-emerald-100 to-teal-200 drop-shadow-[0_0_30px_rgba(52,211,153,0.3)]">
              Compliant Stablecoins
            </span>
          </h1>

          <p className="text-xl md:text-2xl text-emerald-50/90 drop-shadow-md shadow-black max-w-3xl mx-auto mb-12 leading-relaxed animate-slide-up animation-delay-200 font-light">
            Launch compliant stablecoins backed by sovereign bonds.
            Serving the <span className="text-white font-medium">99% of the market</span> beyond USD.
          </p>

          <div className="flex flex-col sm:flex-row gap-6 justify-center items-center animate-slide-up animation-delay-400">
            <GlowingButton size="lg" className="w-full sm:w-auto group" href="/portal/onboarding">
              Start Building
              <ArrowRight className="w-5 h-5 group-hover:translate-x-1 transition-transform" />
            </GlowingButton>
            <GlowingButton variant="outline" size="lg" className="w-full sm:w-auto" href="/docs">
              View Documentation
            </GlowingButton>
          </div>
        </div>
      </section>

      {/* Stats Section - Glass Strip */}
      <section className="relative z-10 py-12 border-y border-white/5 bg-white/[0.02] backdrop-blur-sm">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="grid grid-cols-2 md:grid-cols-4 gap-8">
            {[
              { label: 'Total Value Locked', value: '$10.4M+', icon: TrendingUp },
              { label: 'Currencies Supported', value: '4', icon: Globe },
              { label: 'Reserve Ratio', value: '100%', icon: Shield },
              { label: 'Uptime', value: '99.99%', icon: Zap },
            ].map((stat, i) => (
              <div key={i} className="text-center group">
                <div className="flex justify-center mb-4 text-gray-500 group-hover:text-emerald-400 transition-colors duration-300">
                  <stat.icon className="w-6 h-6" />
                </div>
                <p className="text-3xl md:text-4xl font-heading font-bold text-white mb-2 tracking-tight">{stat.value}</p>
                <p className="text-xs font-medium text-gray-500 uppercase tracking-widest">{stat.label}</p>
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* Features Grid */}
      <section id="features" className="relative z-10 py-32">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="text-center mb-20">
            <h2 className="text-3xl md:text-5xl font-heading font-bold mb-6">
              Enterprise-Grade <span className="text-emerald-400">Infrastructure</span>
            </h2>
            <p className="text-xl text-gray-400 max-w-2xl mx-auto font-light">
              Built for the next generation of financial applications with compliance and security at the core.
            </p>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-3 gap-8">
            <GlassCard className="p-8 group" hoverEffect>
              <div className="w-14 h-14 rounded-2xl bg-emerald-500/10 flex items-center justify-center mb-6 group-hover:scale-110 transition-transform duration-300 border border-emerald-500/20">
                <Globe className="w-7 h-7 text-emerald-400" />
              </div>
              <h3 className="text-2xl font-bold mb-4 text-white">Multi-Currency Support</h3>
              <p className="text-gray-400 leading-relaxed">
                Deploy stablecoins for EUR, GBP, JPY, and emerging markets. Break free from USD dominance.
              </p>
            </GlassCard>

            <GlassCard className="p-8 group" hoverEffect>
              <div className="w-14 h-14 rounded-2xl bg-purple-500/10 flex items-center justify-center mb-6 group-hover:scale-110 transition-transform duration-300 border border-purple-500/20">
                <Code2 className="w-7 h-7 text-purple-400" />
              </div>
              <h3 className="text-2xl font-bold mb-4 text-white">x402 Agent Payments</h3>
              <p className="text-gray-400 leading-relaxed">
                Native AI payment protocols designed for the autonomous agent economy.
              </p>
            </GlassCard>

            <GlassCard className="p-8 group" hoverEffect>
              <div className="w-14 h-14 rounded-2xl bg-blue-500/10 flex items-center justify-center mb-6 group-hover:scale-110 transition-transform duration-300 border border-blue-500/20">
                <Building2 className="w-7 h-7 text-blue-400" />
              </div>
              <h3 className="text-2xl font-bold mb-4 text-white">Regulatory Compliant</h3>
              <p className="text-gray-400 leading-relaxed">
                Built-in compliance for GENIUS Act, MiCA, and global AML/KYC requirements.
              </p>
            </GlassCard>
          </div>
        </div>
      </section>

      {/* CTA Section */}
      <section className="relative z-10 py-32">
        <div className="max-w-5xl mx-auto px-4 sm:px-6 lg:px-8">
          <GlassCard className="p-12 md:p-20 text-center relative overflow-hidden border-emerald-500/20">
            <div className="absolute inset-0 bg-gradient-to-r from-emerald-500/5 via-teal-500/5 to-emerald-500/5" />

            <h2 className="text-4xl md:text-5xl font-heading font-bold mb-8 relative z-10">
              Ready to Launch?
            </h2>
            <p className="text-xl text-gray-400 max-w-2xl mx-auto mb-12 relative z-10 font-light">
              Join the financial revolution. Start building with Meridian's sovereign stablecoin infrastructure today.
            </p>

            <div className="flex flex-col sm:flex-row gap-6 justify-center relative z-10">
              <GlowingButton size="lg" href="/portal/onboarding">Get Started Now</GlowingButton>
              <GlowingButton variant="outline" size="lg" href="/contact">Contact Sales</GlowingButton>
            </div>
          </GlassCard>
        </div>
      </section>

      {/* Footer */}
      <footer className="relative z-10 border-t border-white/5 py-12 bg-[#050608]">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 flex flex-col md:flex-row justify-between items-center gap-6">
          <div className="flex items-center gap-2">
            <div className="w-6 h-6 rounded bg-emerald-500/20 flex items-center justify-center">
              <Globe className="w-4 h-4 text-emerald-400" />
            </div>
            <span className="font-heading font-bold text-gray-300">Meridian</span>
          </div>
          <p className="text-sm text-gray-500">
            © 2025 Meridian Finance. All rights reserved.
          </p>
          <div className="flex gap-4">
            <a
              href="/"
              title="Coming Soon"
              className="min-h-[44px] min-w-[44px] flex items-center justify-center text-gray-500 hover:text-gray-400 transition-colors cursor-default opacity-60"
              aria-label="Twitter (Coming Soon)"
            >
              Twitter
            </a>
            <a
              href="/"
              title="Coming Soon"
              className="min-h-[44px] min-w-[44px] flex items-center justify-center text-gray-500 hover:text-gray-400 transition-colors cursor-default opacity-60"
              aria-label="GitHub (Coming Soon)"
            >
              GitHub
            </a>
            <a
              href="/"
              title="Coming Soon"
              className="min-h-[44px] min-w-[44px] flex items-center justify-center text-gray-500 hover:text-gray-400 transition-colors cursor-default opacity-60"
              aria-label="Discord (Coming Soon)"
            >
              Discord
            </a>
          </div>
        </div>
      </footer>
    </div>
  );
}
