'use client';

import { ProtectedRoute } from '@/lib/auth/AuthContext';
import { OnboardingWizard } from '@/components/portal/OnboardingWizard';

export default function OnboardingPage() {
  return (
    <ProtectedRoute>
      <OnboardingWizard />
    </ProtectedRoute>
  );
}

