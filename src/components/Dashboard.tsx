import { format } from 'date-fns';
import { TimeRing } from './TimeRing';
import { StatusBadge } from './StatusBadge';
import type { TrackerState } from '../hooks/useTracker';

interface Props {
  tracker: TrackerState;
}

function formatHM(secs: number): string {
  const h = Math.floor(secs / 3600);
  const m = Math.floor((secs % 3600) / 60);
  if (h > 0) return `${h}h ${m}m`;
  if (m > 0) return `${m}m`;
  return `${secs}s`;
}

function formatCurrentSession(secs: number): string {
  const h = Math.floor(secs / 3600);
  const m = Math.floor((secs % 3600) / 60);
  const s = secs % 60;
  if (h > 0) return `${h}h ${String(m).padStart(2, '0')}m`;
  if (m > 0) return `${m}m ${String(s).padStart(2, '0')}s`;
  return `${s}s`;
}

export function Dashboard({ tracker }: Props) {
  const today = format(new Date(), 'EEEE, MMMM d');

  const statCards = [
    {
      label: 'Productive Today',
      value: formatHM(tracker.productiveSecs),
      color: '#22C55E',
      bg: 'rgba(34,197,94,0.08)',
      icon: (
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="#22C55E" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
          <polyline points="22 12 18 12 15 21 9 3 6 12 2 12"/>
        </svg>
      ),
    },
    {
      label: 'Idle Today',
      value: formatHM(tracker.idleSecs),
      color: '#64748B',
      bg: 'rgba(100,116,139,0.08)',
      icon: (
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="#64748B" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
          <circle cx="12" cy="12" r="10"/><polyline points="12 6 12 12 16 14"/>
        </svg>
      ),
    },
    {
      label: 'Current Session',
      value: formatCurrentSession(tracker.sessionDurationSecs),
      color: tracker.status === 'productive' ? '#22C55E' : '#64748B',
      bg: tracker.status === 'productive' ? 'rgba(34,197,94,0.08)' : 'rgba(100,116,139,0.08)',
      icon: (
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke={tracker.status === 'productive' ? '#22C55E' : '#64748B'} strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
          <circle cx="12" cy="12" r="10"/><polyline points="12 6 12 12"/>
        </svg>
      ),
    },
  ];

  return (
    <div className="page">
      {/* Header */}
      <div className="page-header">
        <div>
          <h1 className="page-title">Dashboard</h1>
          <p className="page-subtitle">{today}</p>
        </div>
        <StatusBadge status={tracker.status} sessionDurationSecs={tracker.sessionDurationSecs} />
      </div>

      {/* Ring chart */}
      <div className="card ring-card">
        <TimeRing
          productiveSecs={tracker.productiveSecs}
          idleSecs={tracker.idleSecs}
        />

        {/* Legend */}
        <div className="ring-legend">
          <div className="legend-item">
            <span className="legend-dot" style={{ background: '#22C55E' }} />
            <span style={{ color: '#94A3B8', fontSize: 13 }}>Productive</span>
          </div>
          <div className="legend-item">
            <span className="legend-dot" style={{ background: '#1E293B', border: '1px solid #334155' }} />
            <span style={{ color: '#94A3B8', fontSize: 13 }}>Idle</span>
          </div>
        </div>
      </div>

      {/* Stat cards */}
      <div className="stat-grid">
        {statCards.map((card) => (
          <div key={card.label} className="card stat-card" style={{ '--card-accent': card.color, '--card-bg': card.bg } as React.CSSProperties}>
            <div className="stat-card-header">
              <span className="stat-icon" style={{ background: card.bg }}>{card.icon}</span>
              <span className="stat-label">{card.label}</span>
            </div>
            <div className="stat-value" style={{ color: card.color }}>{card.value}</div>
          </div>
        ))}
      </div>
    </div>
  );
}
