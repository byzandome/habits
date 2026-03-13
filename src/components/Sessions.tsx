import { useEffect, useState } from 'react';
import { format, parseISO } from 'date-fns';
import { api } from '../api';
import type { Session } from '../types';

function formatTime(iso: string): string {
  if (!iso) return '—';
  try {
    return format(parseISO(iso), 'HH:mm');
  } catch {
    return '—';
  }
}

function formatDuration(secs: number): string {
  const h = Math.floor(secs / 3600);
  const m = Math.floor((secs % 3600) / 60);
  const s = secs % 60;
  if (h > 0) return `${h}h ${m}m`;
  if (m > 0) return `${m}m ${s}s`;
  return `${s}s`;
}

export function Sessions() {
  const todayStr = format(new Date(), 'yyyy-MM-dd');
  const [date, setDate] = useState(todayStr);
  const [sessions, setSessions] = useState<Session[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    setLoading(true);
    api.getSessionsForDate(date).then((s) => {
      setSessions(s);
      setLoading(false);
    });
  }, [date]);

  return (
    <div className="page">
      <div className="page-header">
        <div>
          <h1 className="page-title">Sessions</h1>
          <p className="page-subtitle">Detailed timeline for a day</p>
        </div>
        <input
          type="date"
          className="date-input"
          value={date}
          max={todayStr}
          onChange={(e) => setDate(e.target.value)}
        />
      </div>

      {loading ? (
        <div className="empty-state">Loading…</div>
      ) : sessions.length === 0 ? (
        <div className="card empty-state">No sessions found for this day.</div>
      ) : (
        <div className="session-list">
          {sessions.map((s, i) => {
            const isProductive = s.session_type === 'productive';
            const ongoing = s.end_time === '';
            return (
              <div key={s.id === -1 ? `inprogress-${i}` : s.id} className="card session-row">
                <div
                  className="session-type-bar"
                  style={{ background: isProductive ? '#22C55E' : '#334155' }}
                />
                <div className="session-content">
                  <div className="session-time-range">
                    {formatTime(s.start_time)}
                    <span style={{ color: '#475569', margin: '0 6px' }}>→</span>
                    {ongoing ? (
                      <span style={{ color: '#22C55E', fontSize: 12 }}>Ongoing</span>
                    ) : (
                      formatTime(s.end_time)
                    )}
                  </div>
                  <div style={{ display: 'flex', alignItems: 'center', gap: 10, marginTop: 4 }}>
                    <span
                      className="type-badge"
                      style={{
                        background: isProductive ? 'rgba(34,197,94,0.12)' : 'rgba(100,116,139,0.12)',
                        color: isProductive ? '#22C55E' : '#64748B',
                      }}
                    >
                      {isProductive ? 'Productive' : 'Idle'}
                    </span>
                    <span style={{ color: '#64748B', fontSize: 13 }}>
                      {formatDuration(s.duration_secs)}
                    </span>
                  </div>
                </div>
              </div>
            );
          })}
        </div>
      )}
    </div>
  );
}
