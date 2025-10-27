'use client';

import { useAuth, ProtectedRoute } from '@/lib/auth/AuthContext';
import { SacredCard } from '@/components/sacred/Card';
import { SacredGrid } from '@/components/sacred/Grid';
import { SacredButton } from '@/components/sacred/Button';
import { Heading, MonoDisplay, Label } from '@/components/sacred/Typography';
import { MetricDisplay } from '@/components/meridian/MetricDisplay';
import { KYCStatus } from '@/lib/auth/types';

export default function PortalDashboard() {
  return (
    <ProtectedRoute>
      <DashboardContent />
    </ProtectedRoute>
  );
}

function DashboardContent() {
  const { user, logout } = useAuth();

  if (!user) return null;

  return (
    <div className="min-h-screen bg-sacred-gray-100">
      {/* Header */}
      <header className="bg-sacred-white border-b border-sacred-gray-200">
        <div className="sacred-container">
          <nav className="flex items-center justify-between h-16">
            <div className="flex items-center space-x-8">
              <a href="/portal/dashboard" className="font-mono text-lg font-medium">
                MERIDIAN
              </a>
              <div className="hidden md:flex items-center space-x-6">
                <a href="/portal/dashboard" className="text-sm font-mono uppercase tracking-wider text-sacred-gray-600 hover:text-sacred-black">
                  Dashboard
                </a>
                <a href="/portal/mint" className="text-sm font-mono uppercase tracking-wider text-sacred-gray-600 hover:text-sacred-black">
                  Mint/Burn
                </a>
                <a href="/portal/compliance" className="text-sm font-mono uppercase tracking-wider text-sacred-gray-600 hover:text-sacred-black">
                  Compliance
                </a>
                <a href="/portal/settings" className="text-sm font-mono uppercase tracking-wider text-sacred-gray-600 hover:text-sacred-black">
                  Settings
                </a>
              </div>
            </div>
            <div className="flex items-center space-x-4">
              <div className="text-right">
                <div className="text-xs font-mono uppercase text-sacred-gray-600">{user.role}</div>
                <div className="text-sm font-mono">{user.organization}</div>
              </div>
              <SacredButton onClick={logout} variant="outline" size="sm">
                Sign Out
              </SacredButton>
            </div>
          </nav>
        </div>
      </header>

      {/* Main Content */}
      <div className="sacred-container py-8">
        {/* KYC Status Banner */}
        {user.kycStatus !== KYCStatus.APPROVED && (
          <div className="mb-8 p-4 bg-amber-50 border border-amber-200 rounded">
            <div className="flex items-center justify-between">
              <div>
                <h3 className="font-mono text-sm uppercase tracking-wider text-amber-800 mb-1">
                  KYC Verification Required
                </h3>
                <p className="text-xs text-amber-700">
                  {user.kycStatus === KYCStatus.NOT_STARTED && 
                    'Complete KYC verification to access mint/burn operations.'}
                  {user.kycStatus === KYCStatus.IN_PROGRESS && 
                    'Your KYC application is in progress. Continue where you left off.'}
                  {user.kycStatus === KYCStatus.PENDING_REVIEW && 
                    'Your KYC application is under review. We will notify you when approved.'}
                  {user.kycStatus === KYCStatus.REJECTED && 
                    'Your KYC application was rejected. Contact support for details.'}
                </p>
              </div>
              {user.kycStatus === KYCStatus.NOT_STARTED && (
                <SacredButton variant="primary" size="sm">
                  <a href="/portal/onboarding">Start KYC →</a>
                </SacredButton>
              )}
              {user.kycStatus === KYCStatus.IN_PROGRESS && (
                <SacredButton variant="primary" size="sm">
                  <a href="/portal/onboarding">Continue KYC →</a>
                </SacredButton>
              )}
            </div>
          </div>
        )}

        {/* Welcome Section */}
        <div className="mb-8">
          <Heading level={1} className="text-3xl mb-2">
            Welcome back, {user.email.split('@')[0]}
          </Heading>
          <p className="text-sacred-gray-600">
            Manage your multi-currency stablecoin operations
          </p>
        </div>

        {/* Key Metrics */}
        <SacredGrid cols={4} gap={4} className="mb-8">
          <SacredCard>
            <MetricDisplay
              label="Total Deposited"
              value="0"
              format="currency"
            />
          </SacredCard>
          
          <SacredCard>
            <MetricDisplay
              label="Total Minted"
              value="0"
              format="currency"
            />
          </SacredCard>
          
          <SacredCard>
            <MetricDisplay
              label="Active Currencies"
              value={0}
              format="number"
            />
          </SacredCard>
          
          <SacredCard>
            <MetricDisplay
              label="KYC Status"
              value={user.kycStatus}
              format="number"
              status={
                user.kycStatus === KYCStatus.APPROVED ? 'healthy' :
                user.kycStatus === KYCStatus.PENDING_REVIEW ? 'warning' :
                'critical'
              }
            />
          </SacredCard>
        </SacredGrid>

        {/* Quick Actions */}
        <div className="grid grid-cols-1 md:grid-cols-2 gap-6 mb-8">
          <SacredCard>
            <Heading level={3} className="text-lg font-mono uppercase mb-4">
              Quick Actions
            </Heading>
            <div className="space-y-3">
              <SacredButton
                variant={user.kycStatus === KYCStatus.APPROVED ? 'primary' : 'outline'}
                fullWidth
                disabled={user.kycStatus !== KYCStatus.APPROVED}
              >
                <a href="/portal/mint" className="block">Mint Stablecoins →</a>
              </SacredButton>
              <SacredButton
                variant="outline"
                fullWidth
                disabled={user.kycStatus !== KYCStatus.APPROVED}
              >
                <a href="/portal/burn" className="block">Burn Stablecoins →</a>
              </SacredButton>
              <SacredButton variant="ghost" fullWidth>
                <a href="/reserves" className="block">View Public Reserves →</a>
              </SacredButton>
            </div>
          </SacredCard>

          <SacredCard>
            <Heading level={3} className="text-lg font-mono uppercase mb-4">
              Account Information
            </Heading>
            <div className="space-y-3">
              <div>
                <Label>Organization</Label>
                <div className="font-mono text-sm mt-1">{user.organization}</div>
              </div>
              <div>
                <Label>Role</Label>
                <div className="font-mono text-sm mt-1">{user.role}</div>
              </div>
              <div>
                <Label>Wallet Address</Label>
                <div className="font-mono text-xs mt-1 break-all text-sacred-gray-600">
                  {user.walletAddress || 'Not connected'}
                </div>
              </div>
              <div>
                <Label>Member Since</Label>
                <div className="font-mono text-sm mt-1">
                  {new Date(user.createdAt).toLocaleDateString()}
                </div>
              </div>
            </div>
          </SacredCard>
        </div>

        {/* Recent Activity */}
        <SacredCard>
          <Heading level={3} className="text-lg font-mono uppercase mb-4">
            Recent Activity
          </Heading>
          <div className="text-center py-12">
            <p className="text-sm text-sacred-gray-500 font-mono">
              No recent activity. Start by minting your first stablecoins.
            </p>
          </div>
        </SacredCard>
      </div>
    </div>
  );
}

