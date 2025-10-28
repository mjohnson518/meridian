'use client';

import { useState } from 'react';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { z } from 'zod';
import { SacredCard } from '../sacred/Card';
import { SacredButton } from '../sacred/Button';
import { SacredGrid } from '../sacred/Grid';
import { Heading, Label } from '../sacred/Typography';
import { EntityType } from '@/lib/auth/types';

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
type DocumentData = z.infer<typeof documentSchema>;
type ComplianceData = z.infer<typeof complianceSchema>;
type WalletData = z.infer<typeof walletSchema>;

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

export function OnboardingWizard() {
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
      description: 'Provide your organization details',
      component: EntityStep,
    },
    {
      id: 'documents',
      title: 'Documentation',
      description: 'Upload required compliance documents',
      component: DocumentStep,
    },
    {
      id: 'compliance',
      title: 'Beneficial Ownership',
      description: 'Declare ultimate beneficial owners',
      component: ComplianceStep,
    },
    {
      id: 'wallet',
      title: 'Wallet Setup',
      description: 'Connect your institutional wallet',
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
    // Submit to backend
    console.log('[KYC] Submitting onboarding data:', formData);
    
    // TODO: POST to /api/v1/kyc/applications
    alert("KYC application submitted! You'll receive confirmation within 24-48 hours.");
    window.location.href = '/portal/dashboard';
  };

  const CurrentStepComponent = steps[currentStep].component;

  return (
    <div className="min-h-screen bg-sacred-gray-100">
      {/* Header */}
      <header className="bg-sacred-white border-b border-sacred-gray-200">
        <div className="sacred-container">
          <div className="flex items-center justify-between h-16">
            <div className="font-mono text-lg font-medium">
              MERIDIAN - KYC Onboarding
            </div>
            <div className="text-xs font-mono uppercase text-sacred-gray-600">
              Step {currentStep + 1} of {steps.length}
            </div>
          </div>
        </div>
      </header>

      {/* Progress Indicator */}
      <div className="bg-sacred-white border-b border-sacred-gray-200">
        <div className="sacred-container py-6">
          <div className="flex items-center justify-between">
            {steps.map((step, index) => (
              <div key={step.id} className="flex-1 flex items-center">
                <div className="flex flex-col items-center flex-1">
                  <div className={`w-10 h-10 rounded-full flex items-center justify-center font-mono text-sm border-2 ${
                    index < currentStep ? 'bg-sacred-black text-sacred-white border-sacred-black' :
                    index === currentStep ? 'bg-sacred-white text-sacred-black border-sacred-black' :
                    'bg-sacred-white text-sacred-gray-400 border-sacred-gray-300'
                  }`}>
                    {index < currentStep ? '✓' : index + 1}
                  </div>
                  <div className="mt-2 text-center">
                    <div className={`text-xs font-mono uppercase tracking-wider ${
                      index <= currentStep ? 'text-sacred-black' : 'text-sacred-gray-400'
                    }`}>
                      {step.title}
                    </div>
                    <div className="text-xs text-sacred-gray-500 mt-1">
                      {step.description}
                    </div>
                  </div>
                </div>
                {index < steps.length - 1 && (
                  <div className={`h-px flex-1 mx-4 ${
                    index < currentStep ? 'bg-sacred-black' : 'bg-sacred-gray-300'
                  }`} />
                )}
              </div>
            ))}
          </div>
        </div>
      </div>

      {/* Step Content */}
      <div className="sacred-container py-8">
        <div className="max-w-3xl mx-auto">
          <CurrentStepComponent
            onNext={handleNext}
            onBack={handleBack}
            data={formData[steps[currentStep].id as keyof typeof formData]}
          />
        </div>
      </div>
    </div>
  );
}

// Step 1: Entity Information
function EntityStep({ onNext, onBack, data }: StepProps) {
  const { register, handleSubmit, formState: { errors } } = useForm<EntityData>({
    resolver: zodResolver(entitySchema),
    defaultValues: data,
  });

  return (
    <SacredCard>
      <form onSubmit={handleSubmit(onNext)} className="space-y-6">
        <Heading level={2} className="text-2xl mb-4">
          Entity Information
        </Heading>

        <SacredGrid cols={2} gap={4}>
          <div className="col-span-2">
            <Label htmlFor="legalName" required>Legal Entity Name</Label>
            <input
              id="legalName"
              {...register('legalName')}
              className="w-full px-4 py-2 border border-sacred-gray-200 rounded font-mono text-sm mt-1 focus:outline-none focus:ring-2 focus:ring-sacred-gray-400"
              placeholder="Acme Corporation Ltd."
            />
            {errors.legalName && (
              <p className="text-xs text-red-600 mt-1">{errors.legalName.message}</p>
            )}
          </div>

          <div>
            <Label htmlFor="registrationNumber" required>Registration Number</Label>
            <input
              id="registrationNumber"
              {...register('registrationNumber')}
              className="w-full px-4 py-2 border border-sacred-gray-200 rounded font-mono text-sm mt-1 focus:outline-none focus:ring-2 focus:ring-sacred-gray-400"
              placeholder="12345678"
            />
            {errors.registrationNumber && (
              <p className="text-xs text-red-600 mt-1">{errors.registrationNumber.message}</p>
            )}
          </div>

          <div>
            <Label htmlFor="jurisdiction" required>Jurisdiction</Label>
            <input
              id="jurisdiction"
              {...register('jurisdiction')}
              className="w-full px-4 py-2 border border-sacred-gray-200 rounded font-mono text-sm mt-1 focus:outline-none focus:ring-2 focus:ring-sacred-gray-400"
              placeholder="Delaware, USA"
            />
            {errors.jurisdiction && (
              <p className="text-xs text-red-600 mt-1">{errors.jurisdiction.message}</p>
            )}
          </div>

          <div className="col-span-2">
            <Label htmlFor="entityType" required>Entity Type</Label>
            <select
              id="entityType"
              {...register('entityType')}
              className="w-full px-4 py-2 border border-sacred-gray-200 rounded font-mono text-sm mt-1 focus:outline-none focus:ring-2 focus:ring-sacred-gray-400"
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
              <p className="text-xs text-red-600 mt-1">{errors.entityType.message}</p>
            )}
          </div>

          <div className="col-span-2">
            <Label htmlFor="businessAddress" required>Business Address</Label>
            <textarea
              id="businessAddress"
              {...register('businessAddress')}
              className="w-full px-4 py-2 border border-sacred-gray-200 rounded font-mono text-sm mt-1 focus:outline-none focus:ring-2 focus:ring-sacred-gray-400"
              placeholder="123 Main St, Suite 100, City, State ZIP"
              rows={3}
            />
            {errors.businessAddress && (
              <p className="text-xs text-red-600 mt-1">{errors.businessAddress.message}</p>
            )}
          </div>

          <div>
            <Label htmlFor="incorporationDate" required>Incorporation Date</Label>
            <input
              id="incorporationDate"
              type="date"
              {...register('incorporationDate')}
              className="w-full px-4 py-2 border border-sacred-gray-200 rounded font-mono text-sm mt-1 focus:outline-none focus:ring-2 focus:ring-sacred-gray-400"
            />
            {errors.incorporationDate && (
              <p className="text-xs text-red-600 mt-1">{errors.incorporationDate.message}</p>
            )}
          </div>

          <div>
            <Label htmlFor="taxId">Tax ID / EIN</Label>
            <input
              id="taxId"
              {...register('taxId')}
              className="w-full px-4 py-2 border border-sacred-gray-200 rounded font-mono text-sm mt-1 focus:outline-none focus:ring-2 focus:ring-sacred-gray-400"
              placeholder="12-3456789"
            />
          </div>
        </SacredGrid>

        <div className="flex justify-end space-x-4 pt-6 border-t border-sacred-gray-200">
          <SacredButton type="submit" variant="primary">
            Continue →
          </SacredButton>
        </div>
      </form>
    </SacredCard>
  );
}

// Step 2: Document Upload
function DocumentStep({ onNext, onBack, data }: StepProps) {
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
    
    // Validate required documents
    if (!files.incorporationDoc || !files.proofOfAddress || !files.bankStatement) {
      alert('Please upload all required documents');
      return;
    }

    onNext(files);
  };

  return (
    <SacredCard>
      <form onSubmit={handleSubmit} className="space-y-6">
        <Heading level={2} className="text-2xl mb-4">
          Documentation
        </Heading>

        <div className="space-y-4">
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
        </div>

        <div className="flex justify-between space-x-4 pt-6 border-t border-sacred-gray-200">
          <SacredButton type="button" variant="outline" onClick={onBack}>
            ← Back
          </SacredButton>
          <SacredButton type="submit" variant="primary">
            Continue →
          </SacredButton>
        </div>
      </form>
    </SacredCard>
  );
}

// File upload component
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
  const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const selectedFile = e.target.files?.[0] || null;
    onChange(selectedFile);
  };

  return (
    <div>
      <Label required={required}>{label}</Label>
      {helpText && (
        <p className="text-xs text-sacred-gray-500 mt-1 mb-2">{helpText}</p>
      )}
      <div className="mt-1">
        <input
          type="file"
          onChange={handleChange}
          accept={accept}
          className="block w-full text-sm text-sacred-gray-600 file:mr-4 file:py-2 file:px-4 file:rounded file:border file:border-sacred-gray-200 file:text-sm file:font-mono file:bg-sacred-white file:text-sacred-black hover:file:bg-sacred-gray-100"
        />
        {file && (
          <div className="mt-2 flex items-center space-x-2 text-xs font-mono text-emerald-600">
            <span>✓</span>
            <span>{file.name} ({(file.size / 1024).toFixed(0)} KB)</span>
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
    setOwners(owners.filter((_, i) => i !== index));
  };

  return (
    <SacredCard>
      <form onSubmit={handleSubmit(onNext)} className="space-y-6">
        <Heading level={2} className="text-2xl mb-4">
          Beneficial Ownership & Compliance
        </Heading>

        <div className="space-y-4">
          <div>
            <Label required>Ultimate Beneficial Owners (25%+ ownership)</Label>
            <p className="text-xs text-sacred-gray-500 mt-1 mb-4">
              List all individuals who own 25% or more of the organization
            </p>

            <div className="space-y-3">
              {owners.map((owner, index) => (
                <div key={index} className="p-4 border border-sacred-gray-200 rounded">
                  <SacredGrid cols={3} gap={3}>
                    <div>
                      <Label>Full Name</Label>
                      <input
                        {...register(`beneficialOwners.${index}.name` as const)}
                        className="w-full px-3 py-2 border border-sacred-gray-200 rounded font-mono text-sm mt-1"
                        placeholder="John Doe"
                      />
                    </div>
                    <div>
                      <Label>Ownership %</Label>
                      <input
                        type="number"
                        {...register(`beneficialOwners.${index}.ownership` as const, { valueAsNumber: true })}
                        className="w-full px-3 py-2 border border-sacred-gray-200 rounded font-mono text-sm mt-1"
                        placeholder="50"
                        min="1"
                        max="100"
                      />
                    </div>
                    <div>
                      <Label>Jurisdiction</Label>
                      <input
                        {...register(`beneficialOwners.${index}.jurisdiction` as const)}
                        className="w-full px-3 py-2 border border-sacred-gray-200 rounded font-mono text-sm mt-1"
                        placeholder="USA"
                      />
                    </div>
                  </SacredGrid>
                  {owners.length > 1 && (
                    <button
                      type="button"
                      onClick={() => removeOwner(index)}
                      className="text-xs text-red-600 font-mono mt-2 hover:underline"
                    >
                      Remove Owner
                    </button>
                  )}
                </div>
              ))}
            </div>

            <button
              type="button"
              onClick={addOwner}
              className="text-sm font-mono text-sacred-black hover:underline mt-2"
            >
              + Add Another Owner
            </button>
          </div>

          <div>
            <Label required>Politically Exposed Person (PEP) Declaration</Label>
            <div className="mt-2 space-y-2">
              <label className="flex items-center space-x-2">
                <input
                  type="checkbox"
                  {...register('isPEP')}
                  className="rounded border-sacred-gray-300"
                />
                <span className="text-sm">
                  Any beneficial owner is a politically exposed person or holds a prominent public position
                </span>
              </label>
            </div>
            {watch('isPEP') && (
              <div className="mt-3">
                <Label>PEP Details</Label>
                <textarea
                  {...register('pepDetails')}
                  className="w-full px-4 py-2 border border-sacred-gray-200 rounded font-mono text-sm mt-1"
                  placeholder="Describe the political position and jurisdiction"
                  rows={3}
                />
              </div>
            )}
          </div>

          <div>
            <Label htmlFor="businessPurpose" required>Business Purpose for Stablecoins</Label>
            <textarea
              id="businessPurpose"
              {...register('businessPurpose')}
              className="w-full px-4 py-2 border border-sacred-gray-200 rounded font-mono text-sm mt-1"
              placeholder="Describe your intended use of Meridian stablecoins (minimum 50 characters)"
              rows={4}
            />
            {errors.businessPurpose && (
              <p className="text-xs text-red-600 mt-1">{errors.businessPurpose.message}</p>
            )}
          </div>

          <div>
            <Label htmlFor="expectedVolume" required>Expected Monthly Volume</Label>
            <select
              id="expectedVolume"
              {...register('expectedVolume')}
              className="w-full px-4 py-2 border border-sacred-gray-200 rounded font-mono text-sm mt-1"
            >
              <option value="">Select range...</option>
              <option value="0-100k">$0 - $100,000</option>
              <option value="100k-1m">$100,000 - $1,000,000</option>
              <option value="1m-10m">$1,000,000 - $10,000,000</option>
              <option value="10m+">$10,000,000+</option>
            </select>
            {errors.expectedVolume && (
              <p className="text-xs text-red-600 mt-1">{errors.expectedVolume.message}</p>
            )}
          </div>
        </div>

        <div className="flex justify-between space-x-4 pt-6 border-t border-sacred-gray-200">
          <SacredButton type="button" variant="outline" onClick={onBack}>
            ← Back
          </SacredButton>
          <SacredButton type="submit" variant="primary">
            Continue →
          </SacredButton>
        </div>
      </form>
    </SacredCard>
  );
}

// Step 4: Wallet Setup
function WalletStep({ onNext, onBack, data }: StepProps) {
  const [connected, setConnected] = useState(false);
  const [walletAddress, setWalletAddress] = useState(data?.walletAddress || '');
  const [walletType, setWalletType] = useState(data?.walletType || 'metamask');

  const handleConnect = async () => {
    // Connect wallet
    if (typeof window.ethereum !== 'undefined') {
      try {
        const accounts = await window.ethereum.request({ method: 'eth_requestAccounts' });
        setWalletAddress(accounts[0]);
        setConnected(true);
      } catch (error) {
        console.error('Wallet connection failed:', error);
      }
    } else {
      // Mock wallet for development
      setWalletAddress('0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb');
      setConnected(true);
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
    <SacredCard>
      <form onSubmit={handleSubmit} className="space-y-6">
        <Heading level={2} className="text-2xl mb-4">
          Wallet Setup
        </Heading>

        <div className="space-y-4">
          <div>
            <Label>Wallet Type</Label>
            <select
              value={walletType}
              onChange={(e) => setWalletType(e.target.value)}
              className="w-full px-4 py-2 border border-sacred-gray-200 rounded font-mono text-sm mt-1"
            >
              <option value="metamask">MetaMask</option>
              <option value="ledger">Ledger Hardware Wallet</option>
              <option value="gnosis-safe">Gnosis Safe (Multi-sig)</option>
              <option value="fireblocks">Fireblocks</option>
            </select>
          </div>

          {!connected ? (
            <div className="p-6 bg-sacred-gray-100 rounded text-center">
              <p className="text-sm text-sacred-gray-600 mb-4 font-mono">
                Connect your institutional wallet to continue
              </p>
              <SacredButton type="button" variant="primary" onClick={handleConnect}>
                Connect Wallet
              </SacredButton>
            </div>
          ) : (
            <div className="p-6 bg-emerald-50 border border-emerald-200 rounded">
              <div className="flex items-center space-x-2 mb-2">
                <div className="w-2 h-2 rounded-full bg-emerald-600"></div>
                <span className="text-xs font-mono uppercase text-emerald-800">
                  Wallet Connected
                </span>
              </div>
              <div className="font-mono text-xs text-emerald-900 break-all">
                {walletAddress}
              </div>
              <button
                type="button"
                onClick={() => setConnected(false)}
                className="text-xs text-emerald-700 font-mono mt-2 hover:underline"
              >
                Disconnect
              </button>
            </div>
          )}

          <div className="p-4 bg-sacred-gray-100 rounded">
            <h4 className="text-xs font-mono uppercase text-sacred-gray-700 mb-2">
              Security Note
            </h4>
            <p className="text-xs text-sacred-gray-600">
              This wallet will be used for all mint/burn operations. Ensure it's controlled by your organization's treasury team and follows your internal security policies.
            </p>
          </div>
        </div>

        <div className="flex justify-between space-x-4 pt-6 border-t border-sacred-gray-200">
          <SacredButton type="button" variant="outline" onClick={onBack}>
            ← Back
          </SacredButton>
          <SacredButton
            type="submit"
            variant="primary"
            disabled={!connected}
          >
            Submit Application →
          </SacredButton>
        </div>
      </form>
    </SacredCard>
  );
}

