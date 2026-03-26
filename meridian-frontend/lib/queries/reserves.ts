import { useQuery } from '@tanstack/react-query';
import { realtimeApi } from '@/lib/api/realtime-client';
import type { ReserveData, AttestationStatus } from '@/lib/api/client';

export const reserveKeys = {
  all: ['reserves'] as const,
  byCurrency: (currency: string) => [...reserveKeys.all, currency] as const,
  attestation: () => [...reserveKeys.all, 'attestation'] as const,
};

export function useReserveQuery(currency: string = 'EUR') {
  return useQuery<ReserveData>({
    queryKey: reserveKeys.byCurrency(currency),
    queryFn: async () => {
      try {
        return await realtimeApi.getReserves(currency);
      } catch {
        // Fallback to mock data when backend is unavailable
        return realtimeApi.getMockReserves();
      }
    },
    staleTime: 30 * 1000,     // 30s — reserve data refreshes every 6h on-chain but we poll faster
    refetchInterval: 60 * 1000, // Re-fetch every 60s
  });
}

export function useAttestationQuery() {
  return useQuery<AttestationStatus>({
    queryKey: reserveKeys.attestation(),
    queryFn: async () => {
      try {
        return await realtimeApi.getAttestationStatus();
      } catch {
        return realtimeApi.getMockAttestation();
      }
    },
    staleTime: 5 * 60 * 1000,  // 5m — attestations happen every 6h
    refetchInterval: 5 * 60 * 1000,
  });
}
