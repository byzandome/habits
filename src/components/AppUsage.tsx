import { useCallback, useEffect, useRef, useState } from 'react';
import { format } from 'date-fns';
import { api } from '../api';
import type { AppUsageStat } from '../types';

function formatHM(secs: number): string {
  const h = Math.floor(secs / 3600);
  const m = Math.floor((secs % 3600) / 60);
  if (h > 0) return `${h}h ${m}m`;
  if (m > 0) return `${m}m`;
  return `${secs}s`;
}

function AppIcon() {
  return (
    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <rect x="2" y="3" width="20" height="14" rx="2"/>
      <line x1="8" y1="21" x2="16" y2="21"/>
      <line x1="12" y1="17" x2="12" y2="21"/>
    </svg>
  );
}

function RefreshIcon() {
  return (
    <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round">
      <polyline points="23 4 23 10 17 10"/>
      <polyline points="1 20 1 14 7 14"/>
      <path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15"/>
    </svg>
  );
}

export function AppUsage() {
  const todayStr = format(new Date(), 'yyyy-MM-dd');
  const [date, setDate] = useState(todayStr);
  const [stats, setStats] = useState<AppUsageStat[]>([]);
  const [loading, setLoading] = useState(true);
  const [refreshing, setRefreshing] = useState(false);
  const intervalRef = useRef<ReturnType<typeof setInterval> | null>(null);

  const fetchData = useCallback((d: string, isRefresh = false) => {
    if (isRefresh) {
      setRefreshing(true);
    } else {
      setLoading(true);
    }
    api.getAppUsage(d).then((data) => {
      setStats(data);
      setLoading(false);
      setRefreshing(false);
    });
  }, []);

  useEffect(() => {
    fetchData(date);
  }, [date, fetchData]);

  // Auto-refresh every 5 min, only when viewing today
  useEffect(() => {
    if (date !== todayStr) return;
    intervalRef.current = setInterval(() => fetchData(date, true), 5 * 60 * 1000);
    return () => {
      if (intervalRef.current) clearInterval(intervalRef.current);
    };
  }, [date, todayStr, fetchData]);

  const totalSecs = stats.reduce((sum, s) => sum + s.duration_secs, 0);
  const maxSecs = stats[0]?.duration_secs ?? 1;

  return (
    <div className="page">
      <div className="page-header">
        <div>
          <h1 className="page-title">App Usage</h1>
          <p className="page-subtitle">Productive time per application</p>
        </div>
        <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
          <button
            className={`refresh-btn${refreshing || loading ? ' refresh-btn--spinning' : ''}`}
            onClick={() => fetchData(date, true)}
            disabled={loading || refreshing}
            title="Refresh"
          >
            <RefreshIcon />
          </button>
          <input
            type="date"
            className="date-input"
            value={date}
            max={todayStr}
            onChange={(e) => setDate(e.target.value)}
          />
        </div>
      </div>

      {/* Summary card */}
      {!loading && stats.length > 0 && (
        <div className="card" style={{ padding: '16px 20px', marginBottom: 12, display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="#22C55E" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <polyline points="22 12 18 12 15 21 9 3 6 12 2 12"/>
            </svg>
            <span style={{ color: '#94A3B8', fontSize: 13 }}>Productive time tracked</span>
          </div>
          <span style={{ color: '#22C55E', fontWeight: 600, fontSize: 15 }}>{formatHM(totalSecs)}</span>
        </div>
      )}

      {loading ? (
        <div className="empty-state">Loading…</div>
      ) : stats.length === 0 ? (
        <div className="card empty-state">No app activity recorded for this day.</div>
      ) : (
        <div style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
          {stats.map((s, i) => {
            const pct = Math.round((s.duration_secs / maxSecs) * 100);
            const share = totalSecs > 0 ? Math.round((s.duration_secs / totalSecs) * 100) : 0;
            return (
              <div key={s.app_name} className="card" style={{ padding: '14px 18px' }}>
                <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: 10 }}>
                  <div style={{ display: 'flex', alignItems: 'center', gap: 10 }}>
                    {/* Rank badge */}
                    <span style={{
                      width: 22,
                      height: 22,
                      borderRadius: 6,
                      background: i === 0 ? 'rgba(34,197,94,0.15)' : 'rgba(100,116,139,0.12)',
                      color: i === 0 ? '#22C55E' : '#64748B',
                      fontSize: 11,
                      fontWeight: 700,
                      display: 'flex',
                      alignItems: 'center',
                      justifyContent: 'center',
                      flexShrink: 0,
                    }}>
                      {i + 1}
                    </span>
                    {/* App icon */}
                    <span style={{ color: '#64748B' }}>
                      <AppIcon />
                    </span>
                    {/* App name */}
                    <span style={{ fontSize: 14, fontWeight: 500, color: '#E2E8F0' }}>
                      {s.app_name}
                    </span>
                  </div>
                  <div style={{ display: 'flex', alignItems: 'center', gap: 10 }}>
                    <span style={{ color: '#64748B', fontSize: 12 }}>{share}%</span>
                    <span style={{ color: '#F8FAFC', fontWeight: 600, fontSize: 14, minWidth: 52, textAlign: 'right' }}>
                      {formatHM(s.duration_secs)}
                    </span>
                  </div>
                </div>
                {/* Progress bar */}
                <div style={{ height: 4, background: '#1E293B', borderRadius: 2, overflow: 'hidden' }}>
                  <div
                    style={{
                      height: '100%',
                      width: `${pct}%`,
                      background: i === 0 ? '#22C55E' : '#334155',
                      borderRadius: 2,
                      transition: 'width 0.3s ease',
                    }}
                  />
                </div>
              </div>
            );
          })}
        </div>
      )}
    </div>
  );
}
