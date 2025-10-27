// API client for Meridian backend

const API_BASE_URL = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8080/api/v1';

export interface ReserveData {
  totalValue: string;
  ratio: number;
  trend: number;
  activeCurrencies: number;
  holdings: BondHolding[];
  history: ReserveHistoryPoint[];
  currencies: CurrencyBreakdown[];
}

export interface BondHolding {
  isin: string;
  name: string;
  maturity: string;
  quantity: number;
  price: number;
  value: number;
  yield: number;
  rating: string;
}

export interface ReserveHistoryPoint {
  timestamp: number;
  ratio: number;
  totalValue: number;
}

export interface CurrencyBreakdown {
  currency: string;
  value: number;
  percentage: number;
}

export interface AttestationStatus {
  timestamp: number;
  status: 'healthy' | 'warning' | 'critical';
  nextAttestation: number;
}

export interface BasketData {
  id: string;
  name: string;
  basketType: 'single' | 'multi' | 'sdr';
  components: BasketComponent[];
  totalValue: string;
  createdAt: string;
}

export interface BasketComponent {
  currencyCode: string;
  targetWeight: string;
  currentWeight: string;
  chainlinkFeed: string;
  lastPrice: string;
}

// Helper for API calls
async function apiCall<T>(
  endpoint: string,
  options?: RequestInit
): Promise<T> {
  const response = await fetch(`${API_BASE_URL}${endpoint}`, {
    ...options,
    headers: {
      'Content-Type': 'application/json',
      ...options?.headers,
    },
  });

  if (!response.ok) {
    throw new Error(`API error: ${response.statusText}`);
  }

  return response.json();
}

// API endpoints
export const api = {
  // Reserve endpoints
  getReserves: () => apiCall<ReserveData>('/reserves'),
  
  getAttestationStatus: () => apiCall<AttestationStatus>('/reserves/attestation'),
  
  // Basket endpoints
  getBaskets: () => apiCall<BasketData[]>('/baskets'),
  
  getBasket: (id: string) => apiCall<BasketData>(`/baskets/${id}`),
  
  createBasket: (data: Partial<BasketData>) =>
    apiCall<BasketData>('/baskets', {
      method: 'POST',
      body: JSON.stringify(data),
    }),
  
  // Oracle endpoints
  getPrice: (currency: string) =>
    apiCall<{ price: string; timestamp: number }>(`/oracle/prices/${currency}`),
  
  updatePrice: (currency: string) =>
    apiCall<{ price: string; updated: boolean }>(`/oracle/prices/${currency}/update`, {
      method: 'POST',
    }),
  
  // Mock data for development
  getMockReserves: (): ReserveData => ({
    totalValue: '10042250.00',
    ratio: 100.42,
    trend: 0.42,
    activeCurrencies: 1,
    holdings: [
      {
        isin: 'DE0001102440',
        name: 'German Bund 2.50% Oct 2027',
        maturity: '2027-10-15',
        quantity: 10050,
        price: 99.50,
        value: 10004750.00,
        yield: 2.65,
        rating: 'AAA',
      },
    ],
    history: Array.from({ length: 30 }, (_, i) => ({
      timestamp: Date.now() - (30 - i) * 86400000,
      ratio: 100 + Math.sin(i / 5) * 2,
      totalValue: 10000000 + Math.sin(i / 5) * 100000,
    })),
    currencies: [
      { currency: 'EUR', value: 10042250.00, percentage: 100 },
    ],
  }),
  
  getMockAttestation: (): AttestationStatus => ({
    timestamp: Date.now() - 3600000, // 1 hour ago
    status: 'healthy',
    nextAttestation: Date.now() + 18000000, // 5 hours from now
  }),
};

export default api;
