'use client';

import { useEffect, useState, useCallback, useRef } from 'react';
import { 
  realtimeApi, 
  connectWebSocket, 
  disconnectWebSocket, 
  subscribeToEvent 
} from '@/lib/api/realtime-client';
import { 
  ReserveData, 
  AttestationStatus, 
  BasketData 
} from '@/lib/api/client';

// Hook for real-time reserve data
export function useRealtimeReserves(currency: string = 'EUR') {
  const [reserves, setReserves] = useState<ReserveData | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [lastUpdate, setLastUpdate] = useState<Date>(new Date());
  
  // Track if component is mounted
  const isMounted = useRef(true);
  
  // Fetch reserves data
  const fetchReserves = useCallback(async () => {
    try {
      const data = await realtimeApi.getReserves(currency);
      if (isMounted.current) {
        setReserves(data);
        setLastUpdate(new Date());
        setError(null);
      }
    } catch (err) {
      if (isMounted.current) {
        setError(err instanceof Error ? err.message : 'Failed to fetch reserves');
        // Use mock data as fallback
        const mockData = realtimeApi.getMockReserves();
        setReserves(mockData);
      }
    } finally {
      if (isMounted.current) {
        setLoading(false);
      }
    }
  }, [currency]);
  
  useEffect(() => {
    isMounted.current = true;
    
    // Initial fetch
    fetchReserves();
    
    // Connect WebSocket
    connectWebSocket();
    
    // Subscribe to reserve updates
    const unsubscribeReserves = subscribeToEvent('reserve_update', (data) => {
      if (isMounted.current && data.currency === currency) {
        setReserves(prev => ({
          ...prev!,
          ...data,
          totalValue: data.total_value || prev?.totalValue || '0',
          ratio: parseFloat(data.reserve_ratio || prev?.ratio || '100')
        }));
        setLastUpdate(new Date());
      }
    });
    
    // Subscribe to price updates
    const unsubscribePrices = subscribeToEvent('price_update', (data) => {
      if (isMounted.current && data.pair === `${currency}/USD`) {
        console.log(`[RT] Price update for ${currency}: ${data.price}`);
        // Update reserves with new price data if needed
        fetchReserves();
      }
    });
    
    // Refresh every 30 seconds (in addition to WebSocket updates)
    const interval = setInterval(fetchReserves, 30000);
    
    return () => {
      isMounted.current = false;
      clearInterval(interval);
      unsubscribeReserves();
      unsubscribePrices();
    };
  }, [currency, fetchReserves]);
  
  return { reserves, loading, error, lastUpdate, refetch: fetchReserves };
}

// Hook for real-time attestation status
export function useRealtimeAttestation() {
  const [attestation, setAttestation] = useState<AttestationStatus | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  
  const isMounted = useRef(true);
  
  const fetchAttestation = useCallback(async () => {
    try {
      const data = await realtimeApi.getAttestationStatus();
      if (isMounted.current) {
        setAttestation(data);
        setError(null);
      }
    } catch (err) {
      if (isMounted.current) {
        setError(err instanceof Error ? err.message : 'Failed to fetch attestation');
        // Use mock data as fallback
        const mockData = realtimeApi.getMockAttestation();
        setAttestation(mockData);
      }
    } finally {
      if (isMounted.current) {
        setLoading(false);
      }
    }
  }, []);
  
  useEffect(() => {
    isMounted.current = true;
    
    // Initial fetch
    fetchAttestation();
    
    // Subscribe to attestation updates
    const unsubscribe = subscribeToEvent('attestation_update', (data) => {
      if (isMounted.current) {
        const timestamp = new Date(data.timestamp).getTime();
        const now = Date.now();
        const hoursSinceAttestation = (now - timestamp) / (1000 * 60 * 60);
        
        setAttestation({
          timestamp,
          status: hoursSinceAttestation < 1 ? 'healthy' : hoursSinceAttestation < 6 ? 'warning' : 'critical',
          nextAttestation: timestamp + (6 * 60 * 60 * 1000)
        });
        
        // Show notification for new attestation
        if ('Notification' in window && Notification.permission === 'granted') {
          new Notification('New Attestation', {
            body: `Reserves attested at ${new Date(timestamp).toLocaleTimeString()}`,
            icon: '/favicon.ico'
          });
        }
      }
    });
    
    // Check attestation status every minute
    const interval = setInterval(fetchAttestation, 60000);
    
    return () => {
      isMounted.current = false;
      clearInterval(interval);
      unsubscribe();
    };
  }, [fetchAttestation]);
  
  return { attestation, loading, error, refetch: fetchAttestation };
}

// Hook for real-time price updates
export function useRealtimePrice(pair: string) {
  const [price, setPrice] = useState<string | null>(null);
  const [timestamp, setTimestamp] = useState<number>(Date.now());
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  
  const isMounted = useRef(true);
  
  const fetchPrice = useCallback(async () => {
    try {
      const data = await realtimeApi.getPrice(pair);
      if (isMounted.current) {
        setPrice(data.price);
        setTimestamp(data.timestamp);
        setError(null);
      }
    } catch (err) {
      if (isMounted.current) {
        setError(err instanceof Error ? err.message : 'Failed to fetch price');
      }
    } finally {
      if (isMounted.current) {
        setLoading(false);
      }
    }
  }, [pair]);
  
  useEffect(() => {
    isMounted.current = true;
    
    // Initial fetch
    fetchPrice();
    
    // Subscribe to price updates for this pair
    const unsubscribe = subscribeToEvent('price_update', (data) => {
      if (isMounted.current && data.pair === pair) {
        setPrice(data.price);
        setTimestamp(data.timestamp || Date.now());
      }
    });
    
    // Refresh every 10 seconds
    const interval = setInterval(fetchPrice, 10000);
    
    return () => {
      isMounted.current = false;
      clearInterval(interval);
      unsubscribe();
    };
  }, [pair, fetchPrice]);
  
  return { price, timestamp, loading, error, refetch: fetchPrice };
}

// Hook to manage WebSocket connection lifecycle
export function useWebSocketConnection() {
  const [connected, setConnected] = useState(false);
  const [reconnecting, setReconnecting] = useState(false);
  
  useEffect(() => {
    // Connect on mount
    connectWebSocket();
    
    // Subscribe to connection events
    const handleConnectionChange = (isConnected: boolean) => {
      setConnected(isConnected);
      setReconnecting(!isConnected);
    };
    
    const unsubscribeOpen = subscribeToEvent('ws_open', () => handleConnectionChange(true));
    const unsubscribeClose = subscribeToEvent('ws_close', () => handleConnectionChange(false));
    
    // Request notification permission for attestation alerts
    if ('Notification' in window && Notification.permission === 'default') {
      Notification.requestPermission();
    }
    
    // Cleanup on unmount
    return () => {
      unsubscribeOpen();
      unsubscribeClose();
      disconnectWebSocket();
    };
  }, []);
  
  return { connected, reconnecting };
}

// Hook for updating prices manually
export function usePriceUpdater() {
  const [updating, setUpdating] = useState(false);
  const [error, setError] = useState<string | null>(null);
  
  const updatePrice = useCallback(async (currency: string) => {
    setUpdating(true);
    setError(null);
    
    try {
      const result = await realtimeApi.updatePrice(currency);
      console.log(`[Price] Updated ${currency}: ${result.price}`);
      return result;
    } catch (err) {
      const message = err instanceof Error ? err.message : 'Failed to update price';
      setError(message);
      throw err;
    } finally {
      setUpdating(false);
    }
  }, []);
  
  return { updatePrice, updating, error };
}
