'use client';

import { useState } from 'react';
import Link from 'next/link';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { z } from 'zod';
import { SacredCard } from '@/components/sacred/Card';
import { SacredButton } from '@/components/sacred/Button';
import { Mail, Building2, Users, MessageSquare, ArrowRight, CheckCircle, Calendar } from 'lucide-react';

const contactSchema = z.object({
  fullName: z.string().min(2, 'Full name is required'),
  email: z.string().email('Please enter a valid email address'),
  company: z.string().min(2, 'Company name is required'),
  companySize: z.string().min(1, 'Please select company size'),
  useCase: z.string().min(1, 'Please select a use case'),
  message: z.string().optional(),
});

type ContactFormData = z.infer<typeof contactSchema>;

const companySizes = [
  { value: '1-10', label: '1-10 employees' },
  { value: '11-50', label: '11-50 employees' },
  { value: '51-200', label: '51-200 employees' },
  { value: '201-1000', label: '201-1000 employees' },
  { value: '1000+', label: '1000+ employees' },
];

const useCases = [
  { value: 'stablecoin-issuance', label: 'Stablecoin Issuance' },
  { value: 'treasury-management', label: 'Treasury Management' },
  { value: 'cross-border-payments', label: 'Cross-border Payments' },
  { value: 'agent-payments', label: 'AI Agent Payments (x402)' },
  { value: 'other', label: 'Other' },
];

export default function ContactPage() {
  const [isSubmitted, setIsSubmitted] = useState(false);
  const [isSubmitting, setIsSubmitting] = useState(false);

  const {
    register,
    handleSubmit,
    formState: { errors },
    reset,
  } = useForm<ContactFormData>({
    resolver: zodResolver(contactSchema),
  });

  const onSubmit = async (data: ContactFormData) => {
    setIsSubmitting(true);

    // Simulate API call
    console.log('Contact form submission:', data);

    // Simulate network delay
    await new Promise(resolve => setTimeout(resolve, 1500));

    setIsSubmitting(false);
    setIsSubmitted(true);
    reset();
  };

  if (isSubmitted) {
    return (
      <div className="min-h-screen bg-[#FAFAFA] dark:bg-[#0A0A0B] flex items-center justify-center px-4">
        <SacredCard className="max-w-md w-full text-center">
          <div className="w-16 h-16 rounded-full bg-emerald-500/10 flex items-center justify-center mx-auto mb-6">
            <CheckCircle className="w-8 h-8 text-emerald-500" />
          </div>
          <h2 className="text-2xl font-medium text-gray-900 dark:text-gray-100 mb-2">
            Message Received
          </h2>
          <p className="text-gray-600 dark:text-gray-400 mb-6">
            Thank you for your interest in Meridian. Our team will review your inquiry and get back to you within 1-2 business days.
          </p>
          <div className="flex flex-col sm:flex-row gap-3 justify-center">
            <Link
              href="/"
              className="inline-flex items-center justify-center gap-2 px-6 py-3 bg-gray-900 dark:bg-white text-white dark:text-gray-900 rounded-lg font-medium hover:bg-gray-800 dark:hover:bg-gray-100 transition-colors"
            >
              Back to Home
            </Link>
            <button
              onClick={() => setIsSubmitted(false)}
              className="inline-flex items-center justify-center gap-2 px-6 py-3 border border-gray-300 dark:border-gray-700 text-gray-900 dark:text-gray-100 rounded-lg font-medium hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors"
            >
              Submit Another
            </button>
          </div>
        </SacredCard>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-[#FAFAFA] dark:bg-[#0A0A0B]">
      <div className="max-w-[1200px] mx-auto px-8 py-16">
        {/* Header */}
        <div className="text-center mb-12">
          <div className="inline-flex items-center justify-center w-16 h-16 rounded-2xl bg-emerald-500/10 mb-6">
            <MessageSquare className="w-8 h-8 text-emerald-500" />
          </div>
          <h1 className="text-4xl md:text-5xl font-medium mb-4 text-gray-900 dark:text-gray-100 tracking-tight">
            Contact Our Team
          </h1>
          <p className="text-lg text-gray-600 dark:text-gray-400 font-light max-w-2xl mx-auto">
            Interested in launching compliant stablecoins? Our team is ready to help you get started.
          </p>
        </div>

        <div className="grid grid-cols-1 lg:grid-cols-3 gap-8">
          {/* Contact Form */}
          <div className="lg:col-span-2">
            <SacredCard>
              <form onSubmit={handleSubmit(onSubmit)} className="space-y-6">
                {/* Name & Email Row */}
                <div className="grid grid-cols-1 sm:grid-cols-2 gap-6">
                  <div>
                    <label htmlFor="fullName" className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                      Full Name <span className="text-red-500">*</span>
                    </label>
                    <input
                      id="fullName"
                      type="text"
                      {...register('fullName')}
                      className={`w-full px-4 py-3 bg-white dark:bg-[#0A0A0B] border rounded-lg text-gray-900 dark:text-gray-100 placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-emerald-500 transition-all ${
                        errors.fullName ? 'border-red-500' : 'border-gray-200 dark:border-gray-700'
                      }`}
                      placeholder="John Smith"
                      aria-invalid={!!errors.fullName}
                      aria-describedby={errors.fullName ? 'fullName-error' : undefined}
                    />
                    {errors.fullName && (
                      <p id="fullName-error" className="mt-1 text-sm text-red-500" role="alert">
                        {errors.fullName.message}
                      </p>
                    )}
                  </div>
                  <div>
                    <label htmlFor="email" className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                      Work Email <span className="text-red-500">*</span>
                    </label>
                    <input
                      id="email"
                      type="email"
                      {...register('email')}
                      className={`w-full px-4 py-3 bg-white dark:bg-[#0A0A0B] border rounded-lg text-gray-900 dark:text-gray-100 placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-emerald-500 transition-all ${
                        errors.email ? 'border-red-500' : 'border-gray-200 dark:border-gray-700'
                      }`}
                      placeholder="john@company.com"
                      aria-invalid={!!errors.email}
                      aria-describedby={errors.email ? 'email-error' : undefined}
                    />
                    {errors.email && (
                      <p id="email-error" className="mt-1 text-sm text-red-500" role="alert">
                        {errors.email.message}
                      </p>
                    )}
                  </div>
                </div>

                {/* Company & Size Row */}
                <div className="grid grid-cols-1 sm:grid-cols-2 gap-6">
                  <div>
                    <label htmlFor="company" className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                      Company Name <span className="text-red-500">*</span>
                    </label>
                    <input
                      id="company"
                      type="text"
                      {...register('company')}
                      className={`w-full px-4 py-3 bg-white dark:bg-[#0A0A0B] border rounded-lg text-gray-900 dark:text-gray-100 placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-emerald-500 transition-all ${
                        errors.company ? 'border-red-500' : 'border-gray-200 dark:border-gray-700'
                      }`}
                      placeholder="Acme Corporation"
                      aria-invalid={!!errors.company}
                      aria-describedby={errors.company ? 'company-error' : undefined}
                    />
                    {errors.company && (
                      <p id="company-error" className="mt-1 text-sm text-red-500" role="alert">
                        {errors.company.message}
                      </p>
                    )}
                  </div>
                  <div>
                    <label htmlFor="companySize" className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                      Company Size <span className="text-red-500">*</span>
                    </label>
                    <select
                      id="companySize"
                      {...register('companySize')}
                      className={`w-full px-4 py-3 bg-white dark:bg-[#0A0A0B] border rounded-lg text-gray-900 dark:text-gray-100 focus:outline-none focus:ring-2 focus:ring-emerald-500 transition-all ${
                        errors.companySize ? 'border-red-500' : 'border-gray-200 dark:border-gray-700'
                      }`}
                      aria-invalid={!!errors.companySize}
                      aria-describedby={errors.companySize ? 'companySize-error' : undefined}
                    >
                      <option value="">Select size...</option>
                      {companySizes.map((size) => (
                        <option key={size.value} value={size.value}>
                          {size.label}
                        </option>
                      ))}
                    </select>
                    {errors.companySize && (
                      <p id="companySize-error" className="mt-1 text-sm text-red-500" role="alert">
                        {errors.companySize.message}
                      </p>
                    )}
                  </div>
                </div>

                {/* Use Case */}
                <div>
                  <label htmlFor="useCase" className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                    Primary Use Case <span className="text-red-500">*</span>
                  </label>
                  <select
                    id="useCase"
                    {...register('useCase')}
                    className={`w-full px-4 py-3 bg-white dark:bg-[#0A0A0B] border rounded-lg text-gray-900 dark:text-gray-100 focus:outline-none focus:ring-2 focus:ring-emerald-500 transition-all ${
                      errors.useCase ? 'border-red-500' : 'border-gray-200 dark:border-gray-700'
                    }`}
                    aria-invalid={!!errors.useCase}
                    aria-describedby={errors.useCase ? 'useCase-error' : undefined}
                  >
                    <option value="">Select use case...</option>
                    {useCases.map((useCase) => (
                      <option key={useCase.value} value={useCase.value}>
                        {useCase.label}
                      </option>
                    ))}
                  </select>
                  {errors.useCase && (
                    <p id="useCase-error" className="mt-1 text-sm text-red-500" role="alert">
                      {errors.useCase.message}
                    </p>
                  )}
                </div>

                {/* Message */}
                <div>
                  <label htmlFor="message" className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                    Message <span className="text-gray-400">(Optional)</span>
                  </label>
                  <textarea
                    id="message"
                    {...register('message')}
                    rows={4}
                    className="w-full px-4 py-3 bg-white dark:bg-[#0A0A0B] border border-gray-200 dark:border-gray-700 rounded-lg text-gray-900 dark:text-gray-100 placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-emerald-500 transition-all resize-none"
                    placeholder="Tell us about your project and requirements..."
                  />
                </div>

                {/* Submit Button */}
                <SacredButton
                  type="submit"
                  variant="primary"
                  fullWidth
                  loading={isSubmitting}
                >
                  {isSubmitting ? 'Sending...' : 'Send Message'}
                </SacredButton>
              </form>
            </SacredCard>
          </div>

          {/* Sidebar */}
          <div className="space-y-6">
            {/* Email Contact */}
            <SacredCard>
              <div className="flex items-start gap-4">
                <div className="w-10 h-10 rounded-lg bg-emerald-500/10 flex items-center justify-center flex-shrink-0">
                  <Mail className="w-5 h-5 text-emerald-500" />
                </div>
                <div>
                  <h3 className="font-medium text-gray-900 dark:text-gray-100 mb-1">
                    Email Us Directly
                  </h3>
                  <p className="text-sm text-gray-600 dark:text-gray-400 mb-2">
                    For general inquiries and support
                  </p>
                  <a
                    href="mailto:sales@meridian.finance"
                    className="text-sm font-mono text-emerald-500 hover:text-emerald-400 transition-colors"
                  >
                    sales@meridian.finance
                  </a>
                </div>
              </div>
            </SacredCard>

            {/* Enterprise */}
            <SacredCard>
              <div className="flex items-start gap-4">
                <div className="w-10 h-10 rounded-lg bg-emerald-500/10 flex items-center justify-center flex-shrink-0">
                  <Building2 className="w-5 h-5 text-emerald-500" />
                </div>
                <div>
                  <h3 className="font-medium text-gray-900 dark:text-gray-100 mb-1">
                    Enterprise Solutions
                  </h3>
                  <p className="text-sm text-gray-600 dark:text-gray-400">
                    Custom implementations for large institutions. Dedicated support and SLAs available.
                  </p>
                </div>
              </div>
            </SacredCard>

            {/* Schedule Demo */}
            <SacredCard>
              <div className="flex items-start gap-4">
                <div className="w-10 h-10 rounded-lg bg-emerald-500/10 flex items-center justify-center flex-shrink-0">
                  <Calendar className="w-5 h-5 text-emerald-500" />
                </div>
                <div>
                  <h3 className="font-medium text-gray-900 dark:text-gray-100 mb-1">
                    Schedule a Demo
                  </h3>
                  <p className="text-sm text-gray-600 dark:text-gray-400 mb-3">
                    See Meridian in action with a personalized walkthrough.
                  </p>
                  <button
                    type="button"
                    className="text-sm font-medium text-emerald-500 hover:text-emerald-400 transition-colors inline-flex items-center gap-1"
                  >
                    Book a time <ArrowRight className="w-4 h-4" />
                  </button>
                </div>
              </div>
            </SacredCard>

            {/* Response Time */}
            <div className="p-4 bg-gray-100 dark:bg-[#141416] rounded-xl border border-gray-200 dark:border-gray-800">
              <div className="flex items-center gap-2 mb-2">
                <Users className="w-4 h-4 text-gray-500" />
                <span className="text-sm font-medium text-gray-700 dark:text-gray-300">
                  Response Time
                </span>
              </div>
              <p className="text-sm text-gray-600 dark:text-gray-400">
                Our team typically responds within 1-2 business days.
              </p>
            </div>
          </div>
        </div>

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
