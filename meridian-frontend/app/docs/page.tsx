'use client';

import Link from 'next/link';
import { SacredCard } from '@/components/sacred/Card';
import { Book, Code, Rocket, Shield, ExternalLink, ArrowRight, FileText, Zap } from 'lucide-react';

export default function DocsPage() {
  const docsSections = [
    {
      title: 'Getting Started',
      description: 'Quick start guide for integrating Meridian stablecoin infrastructure',
      icon: Rocket,
      status: 'coming-soon',
    },
    {
      title: 'API Reference',
      description: 'Complete REST API documentation for minting, burning, and reserve operations',
      icon: Code,
      status: 'coming-soon',
    },
    {
      title: 'SDK Documentation',
      description: 'TypeScript/JavaScript SDK for seamless integration',
      icon: Zap,
      status: 'coming-soon',
    },
    {
      title: 'Security & Compliance',
      description: 'Security practices, audit reports, and compliance frameworks',
      icon: Shield,
      status: 'coming-soon',
    },
  ];

  return (
    <div className="min-h-screen bg-[#FAFAFA] dark:bg-[#0A0A0B]">
      <div className="max-w-[1200px] mx-auto px-8 py-16">
        {/* Header */}
        <div className="text-center mb-16">
          <div className="inline-flex items-center justify-center w-16 h-16 rounded-2xl bg-emerald-500/10 mb-6">
            <Book className="w-8 h-8 text-emerald-500" />
          </div>
          <h1 className="text-4xl md:text-5xl font-medium mb-4 text-gray-900 dark:text-gray-100 tracking-tight">
            Documentation
          </h1>
          <p className="text-lg text-gray-600 dark:text-gray-400 font-light max-w-2xl mx-auto">
            Comprehensive guides and API references for building with Meridian stablecoin infrastructure.
          </p>
          <div className="mt-6 inline-flex items-center gap-2 px-4 py-2 bg-amber-500/10 border border-amber-500/20 rounded-full">
            <div className="w-2 h-2 rounded-full bg-amber-500 animate-pulse" />
            <span className="text-sm font-medium text-amber-600 dark:text-amber-400">
              Documentation Coming Soon
            </span>
          </div>
        </div>

        {/* Docs Grid */}
        <div className="grid grid-cols-1 md:grid-cols-2 gap-6 mb-16">
          {docsSections.map((section, index) => {
            const Icon = section.icon;
            return (
              <SacredCard key={index} className="group relative overflow-hidden">
                <div className="flex items-start gap-4">
                  <div className="flex-shrink-0 w-12 h-12 rounded-xl bg-gray-100 dark:bg-gray-800 flex items-center justify-center group-hover:bg-emerald-500/10 transition-colors">
                    <Icon className="w-6 h-6 text-gray-600 dark:text-gray-400 group-hover:text-emerald-500 transition-colors" />
                  </div>
                  <div className="flex-1">
                    <div className="flex items-center gap-2 mb-1">
                      <h3 className="text-lg font-medium text-gray-900 dark:text-gray-100">
                        {section.title}
                      </h3>
                      <span className="px-2 py-0.5 text-xs font-mono uppercase bg-gray-100 dark:bg-gray-800 text-gray-500 rounded">
                        Soon
                      </span>
                    </div>
                    <p className="text-sm text-gray-600 dark:text-gray-400">
                      {section.description}
                    </p>
                  </div>
                </div>
              </SacredCard>
            );
          })}
        </div>

        {/* GitHub Link */}
        <SacredCard className="text-center">
          <div className="max-w-lg mx-auto">
            <FileText className="w-12 h-12 text-gray-400 mx-auto mb-4" />
            <h2 className="text-xl font-medium text-gray-900 dark:text-gray-100 mb-2">
              View on GitHub
            </h2>
            <p className="text-gray-600 dark:text-gray-400 mb-6">
              While our documentation portal is under construction, you can find setup instructions,
              architecture details, and contribution guidelines in our GitHub repository.
            </p>
            <a
              href="https://github.com/mjohnson518/meridian"
              target="_blank"
              rel="noopener noreferrer"
              className="inline-flex items-center gap-2 px-6 py-3 bg-gray-900 dark:bg-white text-white dark:text-gray-900 rounded-lg font-medium hover:bg-gray-800 dark:hover:bg-gray-100 transition-colors"
            >
              <span>View Repository</span>
              <ExternalLink className="w-4 h-4" />
            </a>
          </div>
        </SacredCard>

        {/* Back to Home */}
        <div className="mt-12 text-center">
          <Link
            href="/"
            className="inline-flex items-center gap-2 text-sm text-gray-500 hover:text-emerald-500 transition-colors"
          >
            <ArrowRight className="w-4 h-4 rotate-180" />
            <span>Back to Home</span>
          </Link>
        </div>
      </div>
    </div>
  );
}
