'use client';

import { AreaChart, Area, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer } from 'recharts';
import { useTheme } from 'next-themes';

interface ReserveDataPoint {
  timestamp: number;
  ratio: number;
  totalValue: number;
}

interface ReserveChartProps {
  data: ReserveDataPoint[];
  className?: string;
}

export function ReserveChart({ data, className = '' }: ReserveChartProps) {
  const { theme } = useTheme();
  const isDark = theme === 'dark';

  const formattedData = data.map(d => ({
    time: new Date(d.timestamp).toLocaleDateString('en-US', { month: 'short', day: 'numeric' }),
    ratio: d.ratio,
    value: d.totalValue,
  }));

  return (
    <div className={className}>
      <ResponsiveContainer width="100%" height={300}>
        <AreaChart data={formattedData} margin={{ top: 10, right: 10, left: 0, bottom: 0 }}>
          <defs>
            <linearGradient id="colorRatio" x1="0" y1="0" x2="0" y2="1">
              <stop offset="5%" stopColor="#10B981" stopOpacity={0.3} />
              <stop offset="95%" stopColor="#10B981" stopOpacity={0} />
            </linearGradient>
          </defs>
          
          <CartesianGrid 
            strokeDasharray="3 3" 
            stroke={isDark ? '#27272A' : '#E5E7EB'}
            vertical={false}
          />
          
          <XAxis
            dataKey="time"
            axisLine={false}
            tickLine={false}
            tick={{ fill: isDark ? '#A1A1AA' : '#6B7280', fontSize: 12, fontFamily: 'IBM Plex Mono' }}
          />
          
          <YAxis
            domain={[98, 104]}
            axisLine={false}
            tickLine={false}
            tick={{ fill: isDark ? '#A1A1AA' : '#6B7280', fontSize: 12, fontFamily: 'IBM Plex Mono' }}
            tickFormatter={(value) => `${value}%`}
          />
          
          <Tooltip
            contentStyle={{
              backgroundColor: isDark ? '#141416' : '#FFFFFF',
              border: `1px solid ${isDark ? '#27272A' : '#E5E7EB'}`,
              borderRadius: '8px',
              fontFamily: 'IBM Plex Mono',
              fontSize: '12px',
            }}
            labelStyle={{ color: isDark ? '#FAFAFA' : '#000000' }}
            formatter={(value: any) => [`${value.toFixed(2)}%`, 'Reserve Ratio']}
          />
          
          <Area
            type="monotone"
            dataKey="ratio"
            stroke="#10B981"
            strokeWidth={2}
            fillOpacity={1}
            fill="url(#colorRatio)"
            animationDuration={1500}
          />
        </AreaChart>
      </ResponsiveContainer>
    </div>
  );
}

