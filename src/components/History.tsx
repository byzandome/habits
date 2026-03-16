import type { DailySummary } from '../types';

import { format, parseISO } from 'date-fns';
import { useEffect, useState } from 'react';
import { BarChart, Bar, XAxis, YAxis, Tooltip, ResponsiveContainer, CartesianGrid } from 'recharts';

import { api } from '../api';

function secsToHours(secs: number): number {
  return Math.round((secs / 3600) * 10) / 10;
}

function formatHM(secs: number): string {
  const h = Math.floor(secs / 3600);
  const m = Math.floor((secs % 3600) / 60);
  if (h > 0) return `${h}h ${m}m`;
  if (m > 0) return `${m}m`;
  return '0m';
}

interface TooltipPayload {
  name: string;
  value: number;
  color: string;
}

function CustomTooltip({
  active,
  payload,
  label,
}: {
  active?: boolean;
  payload?: TooltipPayload[];
  label?: string;
}) {
  if (!active || !payload?.length) return null;
  return (
    <div
      style={{
        background: '#1E1E23',
        border: '1px solid #2A2A2F',
        borderRadius: 8,
        padding: '10px 14px',
        fontSize: 13,
      }}
    >
      <div style={{ color: '#94A3B8', marginBottom: 6, fontWeight: 600 }}>{label}</div>
      {payload.map((p) => (
        <div key={p.name} style={{ color: p.color, marginBottom: 2 }}>
          {p.name}: <strong>{formatHM(p.value)}</strong>
        </div>
      ))}
    </div>
  );
}

export function History() {
  const [data, setData] = useState<DailySummary[]>([]);
  const [loading, setLoading] = useState(true);
  const [days, setDays] = useState(7);

  useEffect(() => {
    setLoading(true);
    api.getHistory(days).then((d) => {
      setData([...d].reverse()); // oldest → newest for chart
      setLoading(false);
    });
  }, [days]);

  const chartData = data.map((d) => ({
    day: format(parseISO(d.date), 'EEE dd'),
    Productive: d.productive_secs,
    Idle: d.idle_secs,
    Locked: d.locked_secs,
    date: d.date,
  }));

  return (
    <div className="page">
      <div className="page-header">
        <div>
          <h1 className="page-title">History</h1>
          <p className="page-subtitle">Your productivity over time</p>
        </div>
        <div className="days-toggle">
          {[7, 14, 30].map((d) => (
            <button
              key={d}
              className={`days-btn ${days === d ? 'days-btn--active' : ''}`}
              onClick={() => setDays(d)}
            >
              {d}d
            </button>
          ))}
        </div>
      </div>

      {/* Bar chart */}
      <div className="card" style={{ padding: '24px 16px 8px' }}>
        {loading ? (
          <div className="empty-state">Loading…</div>
        ) : data.length === 0 ? (
          <div className="empty-state">No history yet. Start using your PC to track time!</div>
        ) : (
          <ResponsiveContainer width="100%" height={240}>
            <BarChart data={chartData} barCategoryGap="30%" barGap={2}>
              <CartesianGrid vertical={false} stroke="#1E293B" />
              <XAxis
                dataKey="day"
                tick={{ fill: '#64748B', fontSize: 12 }}
                axisLine={false}
                tickLine={false}
              />
              <YAxis
                tickFormatter={(v) => `${secsToHours(v)}h`}
                tick={{ fill: '#64748B', fontSize: 12 }}
                axisLine={false}
                tickLine={false}
                width={36}
              />
              <Tooltip content={<CustomTooltip />} cursor={{ fill: 'rgba(255,255,255,0.03)' }} />
              <Bar dataKey="Productive" fill="#22C55E" radius={[4, 4, 0, 0]} maxBarSize={32} />
              <Bar dataKey="Idle" fill="#1E293B" radius={[4, 4, 0, 0]} maxBarSize={32} />
              <Bar dataKey="Locked" fill="#F59E0B" radius={[4, 4, 0, 0]} maxBarSize={32} />
            </BarChart>
          </ResponsiveContainer>
        )}
      </div>

      {/* Day rows */}
      <div className="session-list" style={{ marginTop: 16 }}>
        {[...data].reverse().map((d) => {
          const total = d.productive_secs + d.idle_secs + d.locked_secs;
          const pct = total > 0 ? Math.round((d.productive_secs / total) * 100) : 0;
          return (
            <div key={d.date} className="card history-row">
              <div className="history-row-date">{format(parseISO(d.date), 'EEEE, MMM d')}</div>
              <div className="history-row-stats">
                <span style={{ color: '#22C55E' }}>{formatHM(d.productive_secs)}</span>
                <span style={{ color: '#475569' }}>·</span>
                <span style={{ color: '#64748B' }}>{formatHM(d.idle_secs)} idle</span>
                {d.locked_secs > 0 && (
                  <>
                    <span style={{ color: '#475569' }}>·</span>
                    <span style={{ color: '#F59E0B' }}>{formatHM(d.locked_secs)} locked</span>
                  </>
                )}
                <span
                  className="pct-badge"
                  style={{
                    background: pct >= 70 ? 'rgba(34,197,94,0.15)' : 'rgba(100,116,139,0.15)',
                    color: pct >= 70 ? '#22C55E' : '#64748B',
                  }}
                >
                  {pct}% productive
                </span>
              </div>
              {/* Progress bar */}
              <div className="history-progress-bg">
                <div className="history-progress-fill" style={{ width: `${pct}%` }} />
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
}
