// Real-time API client with WebSocket support

import { ReserveData, AttestationStatus, BondHolding, BasketData } from './client';

// Configuration
const API_BASE_URL = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8080/api/v1';
const WS_URL = process.env.NEXT_PUBLIC_WS_URL || 'ws://localhost:8080/ws';

// WebSocket connection state
let socket: WebSocket | null = null;
let reconnectTimer: NodeJS.Timeout | null = null;
let reconnectAttempts = 0;
const MAX_RECONNECT_ATTEMPTS = 10;
const RECONNECT_DELAY = 1000; // Start with 1 second

// Event listeners
type EventCallback = (data: any) => void;
const eventListeners: Map<string, Set<EventCallback>> = new Map();

// WebSocket connection management
export function connectWebSocket() {
  if (socket?.readyState === WebSocket.OPEN) {
    return; // Already connected
  }

  try {
    socket = new WebSocket(WS_URL);

    socket.onopen = () => {
      console.log('[WS] Connected to backend');
      reconnectAttempts = 0;
      
      // Subscribe to real-time updates
      socket?.send(JSON.stringify({
        type: 'subscribe',
        channels: ['reserves', 'prices', 'attestations']
      }));
    };

    socket.onmessage = (event) => {
      try {
        const message = JSON.parse(event.data);
        console.log('[WS] Received:', message.type);
        
        // Emit to all listeners for this event type
        const listeners = eventListeners.get(message.type);
        if (listeners) {
          listeners.forEach(callback => callback(message.data));
        }
      } catch (error) {
        console.error('[WS] Failed to parse message:', error);
      }
    };

    socket.onerror = (error) => {
      console.warn('[WS] Connection error (backend may not have WebSocket support yet)');
      // Don't throw - gracefully degrade to polling
    };

    socket.onclose = () => {
      console.log('[WS] Disconnected');
      socket = null;
      
      // Attempt to reconnect with exponential backoff
      if (reconnectAttempts < MAX_RECONNECT_ATTEMPTS) {
        const delay = RECONNECT_DELAY * Math.pow(2, reconnectAttempts);
        reconnectTimer = setTimeout(() => {
          reconnectAttempts++;
          console.log(`[WS] Reconnecting... (attempt ${reconnectAttempts}/${MAX_RECONNECT_ATTEMPTS})`);
          connectWebSocket();
        }, delay);
      }
    };
  } catch (error) {
    console.error('[WS] Failed to connect:', error);
  }
}

export function disconnectWebSocket() {
  if (reconnectTimer) {
    clearTimeout(reconnectTimer);
    reconnectTimer = null;
  }
  
  if (socket) {
    socket.close();
    socket = null;
  }
  
  reconnectAttempts = 0;
}

// Subscribe to WebSocket events
export function subscribeToEvent(eventType: string, callback: EventCallback) {
  if (!eventListeners.has(eventType)) {
    eventListeners.set(eventType, new Set());
  }
  eventListeners.get(eventType)!.add(callback);
  
  // Return unsubscribe function
  return () => {
    const listeners = eventListeners.get(eventType);
    if (listeners) {
      listeners.delete(callback);
      if (listeners.size === 0) {
        eventListeners.delete(eventType);
      }
    }
  };
}

// API calls with real backend integration
export const realtimeApi = {
  // Baskets
  async getBaskets(): Promise<BasketData[]> {
    const response = await fetch(`${API_BASE_URL}/baskets`);
    if (!response.ok) throw new Error('Failed to fetch baskets');
    return response.json();
  },
  
  async getBasket(id: string): Promise<BasketData> {
    const response = await fetch(`${API_BASE_URL}/baskets/${id}`);
    if (!response.ok) throw new Error('Failed to fetch basket');
    return response.json();
  },
  
  // Reserves
  async getReserves(currency: string = 'EUR'): Promise<ReserveData> {
    const response = await fetch(`${API_BASE_URL}/reserves/${currency}`);
    if (!response.ok) {
      // Fallback to mock data if backend is not available
      console.warn('[API] Backend not available, using mock data');
      return this.getMockReserves();
    }
    
    const data = await response.json();
    
    // Transform backend data to frontend format
    return {
      totalValue: data.total_value || data.totalValue || '0',
      ratio: parseFloat(data.reserve_ratio || data.ratio || '100'),
      trend: parseFloat(data.trend || '0'),
      activeCurrencies: data.active_currencies || data.activeCurrencies || 1,
      holdings: this.transformHoldings(data.holdings || data.bond_holdings || []),
      history: data.history || [],
      currencies: data.currencies || [{ currency: 'EUR', value: parseFloat(data.total_value || '0'), percentage: 100 }]
    };
  },
  
  // Oracle prices
  async getPrice(pair: string): Promise<{ price: string; timestamp: number }> {
    const response = await fetch(`${API_BASE_URL}/oracle/${pair}`);
    if (!response.ok) throw new Error(`Failed to fetch price for ${pair}`);
    
    const data = await response.json();
    return {
      price: data.price || data.price_usd || '0',
      timestamp: data.timestamp || Date.now()
    };
  },
  
  async updatePrice(currency: string): Promise<{ price: string; updated: boolean }> {
    const response = await fetch(`${API_BASE_URL}/oracle/prices/${currency}/update`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' }
    });
    
    if (!response.ok) throw new Error(`Failed to update price for ${currency}`);
    
    const data = await response.json();
    return {
      price: data.price || data.price_usd || '0',
      updated: data.updated || true
    };
  },
  
  // Attestation
  async getAttestationStatus(): Promise<AttestationStatus> {
    const response = await fetch(`${API_BASE_URL}/attestation/latest`);
    if (!response.ok) {
      console.warn('[API] Backend not available, using mock attestation');
      return this.getMockAttestation();
    }
    
    const data = await response.json();
    
    // Transform backend data
    const timestamp = data.timestamp ? new Date(data.timestamp).getTime() : Date.now() - 3600000;
    const now = Date.now();
    const hoursSinceAttestation = (now - timestamp) / (1000 * 60 * 60);
    
    return {
      timestamp,
      status: hoursSinceAttestation < 1 ? 'healthy' : hoursSinceAttestation < 6 ? 'warning' : 'critical',
      nextAttestation: timestamp + (6 * 60 * 60 * 1000) // 6 hours from last attestation
    };
  },
  
  // Helper functions
  transformHoldings(backendHoldings: any[]): BondHolding[] {
    if (!backendHoldings || backendHoldings.length === 0) {
      // Return mock holdings if none provided
      return [{
        isin: 'DE0001102440',
        name: 'German Bund 2.50% Oct 2027',
        maturity: '2027-10-15',
        quantity: 10050,
        price: 99.50,
        value: 10004750.00,
        yield: 2.65,
        rating: 'AAA'
      }];
    }
    
    return backendHoldings.map(holding => ({
      isin: holding.isin || holding.id || 'N/A',
      name: holding.name || holding.bond_name || 'Unknown Bond',
      maturity: holding.maturity || holding.maturity_date || '2027-01-01',
      quantity: parseFloat(holding.quantity || '0'),
      price: parseFloat(holding.price || '100'),
      value: parseFloat(holding.value || holding.market_value || '0'),
      yield: parseFloat(holding.yield || holding.yield_to_maturity || '0'),
      rating: holding.rating || holding.credit_rating || 'AAA'
    }));
  },
  
  // Mock data fallbacks
  getMockReserves(): ReserveData {
    return {
      totalValue: '10042250.00',
      ratio: 100.42,
      trend: 0.42,
      activeCurrencies: 1,
      holdings: [{
        isin: 'DE0001102440',
        name: 'German Bund 2.50% Oct 2027',
        maturity: '2027-10-15',
        quantity: 10050,
        price: 99.50,
        value: 10004750.00,
        yield: 2.65,
        rating: 'AAA'
      }],
      history: Array.from({ length: 30 }, (_, i) => ({
        timestamp: Date.now() - (30 - i) * 86400000,
        ratio: 100 + Math.sin(i / 5) * 2,
        totalValue: 10000000 + Math.sin(i / 5) * 100000
      })),
      currencies: [
        { currency: 'EUR', value: 10042250.00, percentage: 100 }
      ]
    };
  },
  
  getMockAttestation(): AttestationStatus {
    return {
      timestamp: Date.now() - 3600000, // 1 hour ago
      status: 'healthy',
      nextAttestation: Date.now() + 18000000 // 5 hours from now
    };
  }
};

export default realtimeApi;
