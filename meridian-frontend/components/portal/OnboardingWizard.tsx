'use client';

import { useState } from 'react';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { z } from 'zod';
import { motion, AnimatePresence } from 'framer-motion';
import { PortalCard } from './PortalCard';
import { PortalButton } from './PortalButton';
import { PortalInput, PortalTextarea } from './PortalInput';
import { PortalSelect } from './PortalSelect';
import { EntityType } from '@/lib/auth/types';
import { useAuth } from '@/lib/auth/AuthContext';
import { cn } from '@/lib/utils';

// Validation schemas
const entitySchema = z.object({
  legalName: z.string().min(2, 'Legal name required'),
  registrationNumber: z.string().min(1, 'Registration number required'),
  jurisdiction: z.string().min(2, 'Jurisdiction required'),
  entityType: z.nativeEnum(EntityType),
  businessAddress: z.string().min(10, 'Full address required'),
  incorporationDate: z.string(),
  taxId: z.string().optional(),
});

const documentSchema = z.object({
  incorporationDoc: z.any(),
  proofOfAddress: z.any(),
  bankStatement: z.any(),
  boardResolution: z.any().optional(),
});

const complianceSchema = z.object({
  beneficialOwners: z.array(z.object({
    name: z.string(),
    ownership: z.number().min(1).max(100),
    jurisdiction: z.string(),
  })).min(1),
  isPEP: z.boolean(),
  pepDetails: z.string().optional(),
  businessPurpose: z.string().min(50),
  expectedVolume: z.string(),
});

const walletSchema = z.object({
  walletAddress: z.string().regex(/^0x[a-fA-F0-9]{40}$/, 'Invalid Ethereum address'),
  walletType: z.enum(['metamask', 'ledger', 'gnosis-safe', 'fireblocks']),
  signatureVerified: z.boolean(),
});

type EntityData = z.infer<typeof entitySchema>;
type ComplianceData = z.infer<typeof complianceSchema>;

interface OnboardingStep {
  id: string;
  title: string;
  description: string;
  component: React.ComponentType<StepProps>;
}

interface StepProps {
  onNext: (data: any) => void;
  onBack: () => void;
  data: any;
}

const containerVariants = {
  hidden: { opacity: 0 },
  visible: {
    opacity: 1,
    transition: { staggerChildren: 0.1 },
  },
};

const itemVariants = {
  hidden: { opacity: 0, y: 20 },
  visible: { opacity: 1, y: 0 },
};

const slideVariants = {
  enter: { x: 50, opacity: 0 },
  center: { x: 0, opacity: 1 },
  exit: { x: -50, opacity: 0 },
};

export function OnboardingWizard() {
  const { user } = useAuth();
  const [currentStep, setCurrentStep] = useState(0);
  const [formData, setFormData] = useState({
    entity: {},
    documents: {},
    compliance: {},
    wallet: {},
  });

  const steps: OnboardingStep[] = [
    {
      id: 'entity',
      title: 'Entity Information',
      description: 'Organization details',
      component: EntityStep,
    },
    {
      id: 'documents',
      title: 'Documentation',
      description: 'Compliance documents',
      component: DocumentStep,
    },
    {
      id: 'compliance',
      title: 'Beneficial Ownership',
      description: 'Ultimate owners',
      component: ComplianceStep,
    },
    {
      id: 'wallet',
      title: 'Wallet Setup',
      description: 'Connect wallet',
      component: WalletStep,
    },
  ];

  const handleNext = (stepData: any) => {
    setFormData(prev => ({
      ...prev,
      [steps[currentStep].id]: stepData,
    }));

    if (currentStep < steps.length - 1) {
      setCurrentStep(prev => prev + 1);
    } else {
      handleSubmit();
    }
  };

  const handleBack = () => {
    if (currentStep > 0) {
      setCurrentStep(prev => prev - 1);
    }
  };

  const handleSubmit = async () => {
    try {
      const processedDocuments: any = {};
      const docFiles = formData.documents as Record<string, File | null>;

      for (const [key, file] of Object.entries(docFiles)) {
        if (file) {
          processedDocuments[key] = {
            name: file.name,
            size: file.size,
            type: file.type,
            status: 'uploaded_log'
          };
        }
      }

      if (!user?.id) {
        throw new Error('User not authenticated');
      }
      const userId = parseInt(user.id, 10);

      const payload = {
        user_id: userId,
        entity_info: formData.entity,
        documents: processedDocuments,
        compliance: formData.compliance,
        wallet: formData.wallet
      };

      const { realtimeApi } = await import('@/lib/api/realtime-client');
      const result = await realtimeApi.submitKyc(payload);

      alert("KYC application submitted successfully! Reference ID: " + result.application_id);
      window.location.href = '/portal/dashboard';

    } catch (error: any) {
      console.error('[KYC] Submission failed');
      alert(`Submission failed: ${error.message || 'Please try again'}`);
    }
  };

  const CurrentStepComponent = steps[currentStep].component;

  return (
    <div className="min-h-screen bg-[#050608]">
      {/* Background Grid */}
      <div
        className="fixed inset-0 pointer-events-none opacity-30"
        style={{
          backgroundImage: `linear-gradient(rgba(255,255,255,0.02) 1px, transparent 1px), linear-gradient(90deg, rgba(255,255,255,0.02) 1px, transparent 1px)`,
          backgroundSize: '24px 24px',
          maskImage: 'radial-gradient(ellipse at center, black 0%, transparent 70%)',
        }}
      />

      <div className="relative z-10">
        {/* Header */}
        <header className="bg-[#050608]/80 backdrop-blur-xl border-b border-white/5 sticky top-0 z-50">
          <div className="max-w-7xl mx-auto px-6">
            <div className="flex items-center justify-between h-16">
              <a href="/portal/dashboard" className="font-heading text-lg font-bold">
                <span className="bg-gradient-to-r from-emerald-400 to-teal-500 bg-clip-text text-transparent">
                  MERIDIAN
                </span>
                <span className="text-gray-500 ml-2 font-mono text-sm">KYC</span>
              </a>
              <div className="text-xs font-mono uppercase text-gray-500">
                Step {currentStep + 1} of {steps.length}
              </div>
            </div>
          </div>
        </header>

        {/* Progress Indicator */}
        <div className="bg-[#0B0C10]/50 backdrop-blur-sm border-b border-white/5">
          <div className="max-w-4xl mx-auto px-6 py-8">
            <div className="flex items-center justify-between">
              {steps.map((step, index) => (
                <div key={step.id} className="flex-1 flex items-center">
                  <div className="flex flex-col items-center flex-1">
                    <motion.div
                      className={cn(
                        "w-12 h-12 rounded-full flex items-center justify-center font-mono text-sm border-2 transition-all duration-300",
                        index < currentStep
                          ? 'bg-emerald-500 text-white border-emerald-500 shadow-[0_0_20px_-5px_rgba(16,185,129,0.5)]'
                          : index === currentStep
                            ? 'bg-[#050608] text-emerald-400 border-emerald-500 shadow-[0_0_20px_-5px_rgba(16,185,129,0.3)]'
                            : 'bg-[#050608] text-gray-600 border-white/10'
                      )}
                      initial={{ scale: 0.8 }}
                      animate={{ scale: index === currentStep ? 1.1 : 1 }}
                      transition={{ duration: 0.2 }}
                    >
                      {index < currentStep ? (
                        <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
                        </svg>
                      ) : (
                        index + 1
                      )}
                    </motion.div>
                    <div className="mt-3 text-center">
                      <div className={cn(
                        "text-xs font-mono uppercase tracking-wider transition-colors",
                        index <= currentStep ? 'text-white' : 'text-gray-600'
                      )}>
                        {step.title}
                      </div>
                      <div className="text-xs text-gray-500 mt-1 hidden sm:block">
                        {step.description}
                      </div>
                    </div>
                  </div>
                  {index < steps.length - 1 && (
                    <div className="relative flex-1 mx-4 hidden sm:block">
                      <div className="h-px bg-white/10" />
                      <motion.div
                        className="absolute inset-0 h-px bg-gradient-to-r from-emerald-500 to-teal-500"
                        initial={{ scaleX: 0 }}
                        animate={{ scaleX: index < currentStep ? 1 : 0 }}
                        transition={{ duration: 0.5, delay: 0.2 }}
                        style={{ transformOrigin: 'left' }}
                      />
                    </div>
                  )}
                </div>
              ))}
            </div>
          </div>
        </div>

        {/* Step Content */}
        <motion.div
          className="max-w-3xl mx-auto px-6 py-8"
          variants={containerVariants}
          initial="hidden"
          animate="visible"
        >
          <AnimatePresence mode="wait">
            <motion.div
              key={currentStep}
              variants={slideVariants}
              initial="enter"
              animate="center"
              exit="exit"
              transition={{ duration: 0.3 }}
            >
              <CurrentStepComponent
                onNext={handleNext}
                onBack={handleBack}
                data={formData[steps[currentStep].id as keyof typeof formData]}
              />
            </motion.div>
          </AnimatePresence>
        </motion.div>
      </div>
    </div>
  );
}

// Form input styling helper
const inputClasses = cn(
  "w-full px-4 py-3 rounded-xl font-mono text-sm mt-1",
  "bg-white/[0.02] border border-white/10",
  "text-white placeholder-gray-600",
  "focus:outline-none focus:border-emerald-500/50 focus:ring-2 focus:ring-emerald-500/20",
  "transition-all duration-200"
);

const labelClasses = "text-xs font-mono uppercase tracking-wider text-gray-400 block mb-1";

// Step 1: Entity Information
function EntityStep({ onNext, data }: StepProps) {
  const { register, handleSubmit, formState: { errors } } = useForm<EntityData>({
    resolver: zodResolver(entitySchema),
    defaultValues: data,
  });

  return (
    <PortalCard hoverEffect={false} padding="lg">
      <form onSubmit={handleSubmit(onNext)} className="space-y-6">
        <motion.div variants={itemVariants}>
          <h2 className="text-2xl font-heading font-bold mb-2">
            <span className="bg-gradient-to-r from-white via-gray-200 to-gray-400 bg-clip-text text-transparent">
              Entity Information
            </span>
          </h2>
          <p className="text-sm text-gray-500">
            Provide your organization's legal details
          </p>
        </motion.div>

        <motion.div variants={itemVariants} className="grid grid-cols-1 md:grid-cols-2 gap-4">
          <div className="md:col-span-2">
            <label className={labelClasses}>
              Legal Entity Name <span className="text-emerald-500">*</span>
            </label>
            <input
              {...register('legalName')}
              className={inputClasses}
              placeholder="Acme Corporation Ltd."
            />
            {errors.legalName && (
              <p className="text-xs text-red-400 mt-1">{errors.legalName.message}</p>
            )}
          </div>

          <div>
            <label className={labelClasses}>
              Registration Number <span className="text-emerald-500">*</span>
            </label>
            <input
              {...register('registrationNumber')}
              className={inputClasses}
              placeholder="12345678"
            />
            {errors.registrationNumber && (
              <p className="text-xs text-red-400 mt-1">{errors.registrationNumber.message}</p>
            )}
          </div>

          <div>
            <label className={labelClasses}>
              Jurisdiction <span className="text-emerald-500">*</span>
            </label>
            <input
              {...register('jurisdiction')}
              className={inputClasses}
              placeholder="Delaware, USA"
            />
            {errors.jurisdiction && (
              <p className="text-xs text-red-400 mt-1">{errors.jurisdiction.message}</p>
            )}
          </div>

          <div className="md:col-span-2">
            <label className={labelClasses}>
              Entity Type <span className="text-emerald-500">*</span>
            </label>
            <select
              {...register('entityType')}
              className={inputClasses}
            >
              <option value="">Select entity type...</option>
              <option value={EntityType.CORPORATION}>Corporation</option>
              <option value={EntityType.LLC}>LLC</option>
              <option value={EntityType.TRUST}>Trust</option>
              <option value={EntityType.PARTNERSHIP}>Partnership</option>
              <option value={EntityType.SOVEREIGN}>Sovereign Entity</option>
              <option value={EntityType.FINANCIAL_INSTITUTION}>Financial Institution</option>
            </select>
            {errors.entityType && (
              <p className="text-xs text-red-400 mt-1">{errors.entityType.message}</p>
            )}
          </div>

          <div className="md:col-span-2">
            <label className={labelClasses}>
              Business Address <span className="text-emerald-500">*</span>
            </label>
            <textarea
              {...register('businessAddress')}
              className={cn(inputClasses, "resize-none")}
              placeholder="123 Main St, Suite 100, City, State ZIP"
              rows={3}
            />
            {errors.businessAddress && (
              <p className="text-xs text-red-400 mt-1">{errors.businessAddress.message}</p>
            )}
          </div>

          <div>
            <label className={labelClasses}>
              Incorporation Date <span className="text-emerald-500">*</span>
            </label>
            <input
              type="date"
              {...register('incorporationDate')}
              className={inputClasses}
            />
            {errors.incorporationDate && (
              <p className="text-xs text-red-400 mt-1">{errors.incorporationDate.message}</p>
            )}
          </div>

          <div>
            <label className={labelClasses}>Tax ID / EIN</label>
            <input
              {...register('taxId')}
              className={inputClasses}
              placeholder="12-3456789"
            />
          </div>
        </motion.div>

        <motion.div variants={itemVariants} className="flex justify-end pt-6 border-t border-white/5">
          <PortalButton type="submit" variant="primary">
            Continue
            <svg className="w-4 h-4 ml-2" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 7l5 5m0 0l-5 5m5-5H6" />
            </svg>
          </PortalButton>
        </motion.div>
      </form>
    </PortalCard>
  );
}

// Step 2: Document Upload
function DocumentStep({ onNext, onBack }: StepProps) {
  const [files, setFiles] = useState<Record<string, File | null>>({
    incorporationDoc: null,
    proofOfAddress: null,
    bankStatement: null,
    boardResolution: null,
  });

  const handleFileChange = (key: string, file: File | null) => {
    setFiles(prev => ({ ...prev, [key]: file }));
  };

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();

    if (!files.incorporationDoc || !files.proofOfAddress || !files.bankStatement) {
      alert('Please upload all required documents');
      return;
    }

    onNext(files);
  };

  return (
    <PortalCard hoverEffect={false} padding="lg">
      <form onSubmit={handleSubmit} className="space-y-6">
        <motion.div variants={itemVariants}>
          <h2 className="text-2xl font-heading font-bold mb-2">
            <span className="bg-gradient-to-r from-white via-gray-200 to-gray-400 bg-clip-text text-transparent">
              Documentation
            </span>
          </h2>
          <p className="text-sm text-gray-500">
            Upload required compliance documents
          </p>
        </motion.div>

        <motion.div variants={itemVariants} className="space-y-4">
          <FileUploadField
            label="Certificate of Incorporation"
            required
            file={files.incorporationDoc}
            onChange={(file) => handleFileChange('incorporationDoc', file)}
            accept=".pdf,.jpg,.png"
          />

          <FileUploadField
            label="Proof of Business Address"
            required
            file={files.proofOfAddress}
            onChange={(file) => handleFileChange('proofOfAddress', file)}
            accept=".pdf,.jpg,.png"
            helpText="Utility bill or lease agreement (less than 3 months old)"
          />

          <FileUploadField
            label="Bank Statement"
            required
            file={files.bankStatement}
            onChange={(file) => handleFileChange('bankStatement', file)}
            accept=".pdf"
            helpText="Recent bank statement showing business account"
          />

          <FileUploadField
            label="Board Resolution (if applicable)"
            file={files.boardResolution}
            onChange={(file) => handleFileChange('boardResolution', file)}
            accept=".pdf"
            helpText="Authorizing digital asset operations"
          />
        </motion.div>

        <motion.div variants={itemVariants} className="flex justify-between pt-6 border-t border-white/5">
          <PortalButton type="button" variant="outline" onClick={onBack}>
            <svg className="w-4 h-4 mr-2" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M11 17l-5-5m0 0l5-5m-5 5h12" />
            </svg>
            Back
          </PortalButton>
          <PortalButton type="submit" variant="primary">
            Continue
            <svg className="w-4 h-4 ml-2" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 7l5 5m0 0l-5 5m5-5H6" />
            </svg>
          </PortalButton>
        </motion.div>
      </form>
    </PortalCard>
  );
}

// Dark-themed file upload component
function FileUploadField({
  label,
  required,
  file,
  onChange,
  accept,
  helpText,
}: {
  label: string;
  required?: boolean;
  file: File | null;
  onChange: (file: File | null) => void;
  accept?: string;
  helpText?: string;
}) {
  const [isDragOver, setIsDragOver] = useState(false);

  const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const selectedFile = e.target.files?.[0] || null;
    onChange(selectedFile);
  };

  const handleDrop = (e: React.DragEvent) => {
    e.preventDefault();
    setIsDragOver(false);
    const droppedFile = e.dataTransfer.files[0];
    if (droppedFile) {
      onChange(droppedFile);
    }
  };

  return (
    <div>
      <label className={labelClasses}>
        {label} {required && <span className="text-emerald-500">*</span>}
      </label>
      {helpText && (
        <p className="text-xs text-gray-600 mt-1 mb-2">{helpText}</p>
      )}
      <div
        className={cn(
          "relative mt-1 p-6 rounded-xl border-2 border-dashed transition-all duration-200",
          isDragOver
            ? 'border-emerald-500/50 bg-emerald-500/5'
            : file
              ? 'border-emerald-500/30 bg-emerald-500/5'
              : 'border-white/10 bg-white/[0.02] hover:border-white/20'
        )}
        onDragOver={(e) => { e.preventDefault(); setIsDragOver(true); }}
        onDragLeave={() => setIsDragOver(false)}
        onDrop={handleDrop}
      >
        <input
          type="file"
          onChange={handleChange}
          accept={accept}
          className="absolute inset-0 w-full h-full opacity-0 cursor-pointer"
        />
        {file ? (
          <div className="flex items-center justify-center gap-3">
            <svg className="w-5 h-5 text-emerald-500" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
            </svg>
            <span className="text-sm font-mono text-emerald-400">{file.name}</span>
            <span className="text-xs text-gray-500">({(file.size / 1024).toFixed(0)} KB)</span>
          </div>
        ) : (
          <div className="text-center">
            <svg className="w-8 h-8 mx-auto text-gray-600 mb-2" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M15 13l-3-3m0 0l-3 3m3-3v12" />
            </svg>
            <p className="text-sm text-gray-500 font-mono">
              Drop file here or <span className="text-emerald-400">browse</span>
            </p>
          </div>
        )}
      </div>
    </div>
  );
}

// Step 3: Compliance & Beneficial Ownership
function ComplianceStep({ onNext, onBack, data }: StepProps) {
  const { register, handleSubmit, formState: { errors }, watch } = useForm<ComplianceData>({
    resolver: zodResolver(complianceSchema),
    defaultValues: data || {
      beneficialOwners: [{ name: '', ownership: 0, jurisdiction: '' }],
      isPEP: false,
    },
  });

  const [owners, setOwners] = useState(data?.beneficialOwners || [{ name: '', ownership: 0, jurisdiction: '' }]);

  const addOwner = () => {
    setOwners([...owners, { name: '', ownership: 0, jurisdiction: '' }]);
  };

  const removeOwner = (index: number) => {
    const newOwners = [...owners];
    newOwners.splice(index, 1);
    setOwners(newOwners);
  };

  return (
    <PortalCard hoverEffect={false} padding="lg">
      <form onSubmit={handleSubmit(onNext)} className="space-y-6">
        <motion.div variants={itemVariants}>
          <h2 className="text-2xl font-heading font-bold mb-2">
            <span className="bg-gradient-to-r from-white via-gray-200 to-gray-400 bg-clip-text text-transparent">
              Beneficial Ownership & Compliance
            </span>
          </h2>
          <p className="text-sm text-gray-500">
            Declare ultimate beneficial owners and compliance information
          </p>
        </motion.div>

        <motion.div variants={itemVariants} className="space-y-6">
          {/* Beneficial Owners */}
          <div>
            <label className={labelClasses}>
              Ultimate Beneficial Owners (25%+ ownership) <span className="text-emerald-500">*</span>
            </label>
            <p className="text-xs text-gray-600 mt-1 mb-4">
              List all individuals who own 25% or more of the organization
            </p>

            <div className="space-y-3">
              {owners.map((owner: { name: string; ownership: number; jurisdiction: string }, index: number) => (
                <motion.div
                  key={index}
                  initial={{ opacity: 0, y: 10 }}
                  animate={{ opacity: 1, y: 0 }}
                  className="p-4 rounded-xl bg-white/[0.02] border border-white/10"
                >
                  <div className="grid grid-cols-1 md:grid-cols-3 gap-3">
                    <div>
                      <label className="text-xs font-mono text-gray-500">Full Name</label>
                      <input
                        {...register(`beneficialOwners.${index}.name` as const)}
                        className={cn(inputClasses, "py-2 text-sm")}
                        placeholder="John Doe"
                      />
                    </div>
                    <div>
                      <label className="text-xs font-mono text-gray-500">Ownership %</label>
                      <input
                        type="number"
                        {...register(`beneficialOwners.${index}.ownership` as const, { valueAsNumber: true })}
                        className={cn(inputClasses, "py-2 text-sm")}
                        placeholder="50"
                        min="1"
                        max="100"
                      />
                    </div>
                    <div>
                      <label className="text-xs font-mono text-gray-500">Jurisdiction</label>
                      <input
                        {...register(`beneficialOwners.${index}.jurisdiction` as const)}
                        className={cn(inputClasses, "py-2 text-sm")}
                        placeholder="USA"
                      />
                    </div>
                  </div>
                  {owners.length > 1 && (
                    <button
                      type="button"
                      onClick={() => removeOwner(index)}
                      className="text-xs text-red-400 font-mono mt-3 hover:text-red-300 transition-colors"
                    >
                      Remove Owner
                    </button>
                  )}
                </motion.div>
              ))}
            </div>

            <button
              type="button"
              onClick={addOwner}
              className="text-sm font-mono text-emerald-400 hover:text-emerald-300 mt-3 transition-colors"
            >
              + Add Another Owner
            </button>
          </div>

          {/* PEP Declaration */}
          <div className="p-4 rounded-xl bg-white/[0.02] border border-white/10">
            <label className={labelClasses}>
              Politically Exposed Person (PEP) Declaration <span className="text-emerald-500">*</span>
            </label>
            <div className="mt-3">
              <label className="flex items-center gap-3 cursor-pointer">
                <div className="relative">
                  <input
                    type="checkbox"
                    {...register('isPEP')}
                    className="sr-only peer"
                  />
                  <div className="w-5 h-5 border-2 border-white/20 rounded peer-checked:bg-emerald-500 peer-checked:border-emerald-500 transition-colors" />
                  <svg className="absolute inset-0 w-5 h-5 text-white opacity-0 peer-checked:opacity-100 transition-opacity" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={3} d="M5 13l4 4L19 7" />
                  </svg>
                </div>
                <span className="text-sm text-gray-300">
                  Any beneficial owner is a politically exposed person
                </span>
              </label>
            </div>
            {watch('isPEP') && (
              <motion.div
                initial={{ opacity: 0, height: 0 }}
                animate={{ opacity: 1, height: 'auto' }}
                className="mt-4"
              >
                <label className="text-xs font-mono text-gray-500">PEP Details</label>
                <textarea
                  {...register('pepDetails')}
                  className={cn(inputClasses, "resize-none")}
                  placeholder="Describe the political position and jurisdiction"
                  rows={3}
                />
              </motion.div>
            )}
          </div>

          {/* Business Purpose */}
          <div>
            <label className={labelClasses}>
              Business Purpose for Stablecoins <span className="text-emerald-500">*</span>
            </label>
            <textarea
              {...register('businessPurpose')}
              className={cn(inputClasses, "resize-none")}
              placeholder="Describe your intended use of Meridian stablecoins (minimum 50 characters)"
              rows={4}
            />
            {errors.businessPurpose && (
              <p className="text-xs text-red-400 mt-1">{errors.businessPurpose.message}</p>
            )}
          </div>

          {/* Expected Volume */}
          <div>
            <label className={labelClasses}>
              Expected Monthly Volume <span className="text-emerald-500">*</span>
            </label>
            <select {...register('expectedVolume')} className={inputClasses}>
              <option value="">Select range...</option>
              <option value="0-100k">$0 - $100,000</option>
              <option value="100k-1m">$100,000 - $1,000,000</option>
              <option value="1m-10m">$1,000,000 - $10,000,000</option>
              <option value="10m+">$10,000,000+</option>
            </select>
            {errors.expectedVolume && (
              <p className="text-xs text-red-400 mt-1">{errors.expectedVolume.message}</p>
            )}
          </div>
        </motion.div>

        <motion.div variants={itemVariants} className="flex justify-between pt-6 border-t border-white/5">
          <PortalButton type="button" variant="outline" onClick={onBack}>
            <svg className="w-4 h-4 mr-2" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M11 17l-5-5m0 0l5-5m-5 5h12" />
            </svg>
            Back
          </PortalButton>
          <PortalButton type="submit" variant="primary">
            Continue
            <svg className="w-4 h-4 ml-2" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 7l5 5m0 0l-5 5m5-5H6" />
            </svg>
          </PortalButton>
        </motion.div>
      </form>
    </PortalCard>
  );
}

// Step 4: Wallet Setup
function WalletStep({ onNext, onBack, data }: StepProps) {
  const [connected, setConnected] = useState(false);
  const [walletAddress, setWalletAddress] = useState(data?.walletAddress || '');
  const [walletType, setWalletType] = useState(data?.walletType || 'metamask');
  const [connecting, setConnecting] = useState(false);

  const handleConnect = async () => {
    setConnecting(true);
    try {
      if (typeof window.ethereum !== 'undefined') {
        const accounts = await window.ethereum.request({ method: 'eth_requestAccounts' });
        setWalletAddress(accounts[0]);
        setConnected(true);
      } else {
        // Mock wallet for development
        await new Promise(resolve => setTimeout(resolve, 1000));
        setWalletAddress('0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb');
        setConnected(true);
      }
    } catch (error) {
      console.error('Wallet connection failed:', error);
    } finally {
      setConnecting(false);
    }
  };

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();

    if (!connected || !walletAddress) {
      alert('Please connect your wallet first');
      return;
    }

    onNext({
      walletAddress,
      walletType,
      signatureVerified: connected,
    });
  };

  return (
    <PortalCard hoverEffect={false} padding="lg">
      <form onSubmit={handleSubmit} className="space-y-6">
        <motion.div variants={itemVariants}>
          <h2 className="text-2xl font-heading font-bold mb-2">
            <span className="bg-gradient-to-r from-white via-gray-200 to-gray-400 bg-clip-text text-transparent">
              Wallet Setup
            </span>
          </h2>
          <p className="text-sm text-gray-500">
            Connect your institutional wallet to complete setup
          </p>
        </motion.div>

        <motion.div variants={itemVariants} className="space-y-6">
          {/* Wallet Type Selection */}
          <div>
            <label className={labelClasses}>Wallet Type</label>
            <div className="grid grid-cols-2 gap-3 mt-2">
              {[
                { value: 'metamask', label: 'MetaMask', icon: '🦊' },
                { value: 'ledger', label: 'Ledger', icon: '🔒' },
                { value: 'gnosis-safe', label: 'Gnosis Safe', icon: '🏛' },
                { value: 'fireblocks', label: 'Fireblocks', icon: '🔥' },
              ].map((wallet) => (
                <button
                  key={wallet.value}
                  type="button"
                  onClick={() => setWalletType(wallet.value)}
                  className={cn(
                    "p-4 rounded-xl border font-mono text-sm transition-all duration-200",
                    walletType === wallet.value
                      ? 'bg-emerald-500/10 border-emerald-500/50 text-emerald-400'
                      : 'bg-white/[0.02] border-white/10 text-gray-400 hover:border-white/20'
                  )}
                >
                  <span className="text-xl mr-2">{wallet.icon}</span>
                  {wallet.label}
                </button>
              ))}
            </div>
          </div>

          {/* Connect Wallet */}
          {!connected ? (
            <div className="p-8 rounded-xl bg-white/[0.02] border border-white/10 text-center">
              <div className="w-16 h-16 mx-auto mb-4 rounded-full bg-emerald-500/10 border border-emerald-500/30 flex items-center justify-center">
                <svg className="w-8 h-8 text-emerald-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M13.828 10.172a4 4 0 00-5.656 0l-4 4a4 4 0 105.656 5.656l1.102-1.101m-.758-4.899a4 4 0 005.656 0l4-4a4 4 0 00-5.656-5.656l-1.1 1.1" />
                </svg>
              </div>
              <p className="text-sm text-gray-500 mb-4 font-mono">
                Connect your institutional wallet to continue
              </p>
              <PortalButton
                type="button"
                variant="primary"
                onClick={handleConnect}
                loading={connecting}
              >
                {connecting ? 'Connecting...' : 'Connect Wallet'}
              </PortalButton>
            </div>
          ) : (
            <motion.div
              initial={{ opacity: 0, scale: 0.95 }}
              animate={{ opacity: 1, scale: 1 }}
              className="p-6 rounded-xl bg-emerald-500/10 border border-emerald-500/30"
            >
              <div className="flex items-center gap-2 mb-3">
                <div className="w-3 h-3 rounded-full bg-emerald-500 animate-pulse" />
                <span className="text-xs font-mono uppercase text-emerald-400">
                  Wallet Connected
                </span>
              </div>
              <div className="font-mono text-sm text-emerald-300 break-all mb-3">
                {walletAddress}
              </div>
              <button
                type="button"
                onClick={() => {
                  setConnected(false);
                  setWalletAddress('');
                }}
                className="text-xs text-emerald-400/70 font-mono hover:text-emerald-300 transition-colors"
              >
                Disconnect
              </button>
            </motion.div>
          )}

          {/* Security Note */}
          <div className="p-4 rounded-xl bg-white/[0.02] border border-white/5">
            <h4 className="text-xs font-mono uppercase text-gray-400 mb-2 flex items-center gap-2">
              <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
              </svg>
              Security Note
            </h4>
            <p className="text-xs text-gray-500">
              This wallet will be used for all mint/burn operations. Ensure it's controlled by your organization's treasury team and follows your internal security policies.
            </p>
          </div>
        </motion.div>

        <motion.div variants={itemVariants} className="flex justify-between pt-6 border-t border-white/5">
          <PortalButton type="button" variant="outline" onClick={onBack}>
            <svg className="w-4 h-4 mr-2" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M11 17l-5-5m0 0l5-5m-5 5h12" />
            </svg>
            Back
          </PortalButton>
          <PortalButton
            type="submit"
            variant="primary"
            disabled={!connected}
          >
            Submit Application
            <svg className="w-4 h-4 ml-2" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
            </svg>
          </PortalButton>
        </motion.div>
      </form>
    </PortalCard>
  );
}
