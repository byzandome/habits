import type { Interval, Session } from '@/domain/entities';

import { format, parseISO } from 'date-fns';
import { useEffect, useState } from 'react';

import { sessionUseCases } from '@/infrastructure/container';

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
  const [session, setSession] = useState<Session | null>(null);
  const [intervals, setIntervals] = useState<Interval[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    setLoading(true);
    sessionUseCases.getSessionForDate(date).then((s) => {
      setSession(s);
      if (s?.id) {
        sessionUseCases.getIntervalsForSession(s.id).then(setIntervals);
      } else {
        setIntervals([]);
      }
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
      ) : !session ? (
        <div className="card empty-state">No session found for this day.</div>
      ) : (
        <div className="session-list">
          {/* Session summary card */}
          <div className="card session-row">
            <div className="session-type-bar" style={{ background: '#3B82F6' }} />
            <div className="session-content">
              <div className="session-time-range">
                {formatTime(session.start_time)}
                <span style={{ color: '#475569', margin: '0 6px' }}>→</span>
                {session.end_time === '' ? (
                  <span style={{ color: '#3B82F6', fontSize: 12 }}>Ongoing</span>
                ) : (
                  formatTime(session.end_time)
                )}
                {session.active_secs + session.idle_secs > 0 && (
                  <span style={{ color: '#64748B', fontSize: 12, marginLeft: 8 }}>
                    ({formatDuration(session.active_secs + session.idle_secs)})
                  </span>
                )}
              </div>
              <div style={{ display: 'flex', alignItems: 'center', gap: 10, marginTop: 4 }}>
                <span
                  className="type-badge"
                  style={{ background: 'rgba(34,197,94,0.12)', color: '#22C55E' }}
                >
                  Active: {formatDuration(session.active_secs)}
                </span>
                {session.idle_secs > 0 && (
                  <span
                    className="type-badge"
                    style={{ background: 'rgba(100,116,139,0.12)', color: '#64748B' }}
                  >
                    Idle: {formatDuration(session.idle_secs)}
                  </span>
                )}
                {session.locked_secs > 0 && (
                  <span
                    className="type-badge"
                    style={{ background: 'rgba(245,158,11,0.12)', color: '#F59E0B' }}
                  >
                    Locked: {formatDuration(session.locked_secs)}
                  </span>
                )}
              </div>
            </div>
          </div>

          {/* Intervals */}
          {intervals.length > 0 && (
            <div style={{ marginTop: 12 }}>
              <h3 style={{ fontSize: 13, color: '#64748B', marginBottom: 8 }}>Intervals</h3>
              {intervals.map((iv) => (
                <div key={iv.id} className="card session-row" style={{ marginBottom: 6 }}>
                  <div
                    className="session-type-bar"
                    style={{
                      background:
                        iv.type === 'active'
                          ? '#22C55E'
                          : iv.type === 'idle'
                            ? '#64748B'
                            : iv.type === 'locked'
                              ? '#F59E0B'
                              : '#94A3B8',
                    }}
                  />
                  <div className="session-content">
                    <div className="session-time-range">
                      {formatTime(iv.start_time)}
                      {iv.end_time && (
                        <>
                          <span style={{ color: '#475569', margin: '0 6px' }}>→</span>
                          {formatTime(iv.end_time)}
                        </>
                      )}
                      <span className="type-badge" style={{ marginLeft: 10 }}>
                        {iv.type}
                      </span>
                    </div>
                    <div style={{ fontSize: 12, color: '#94A3B8', marginTop: 2 }}>
                      {formatDuration(iv.duration_secs)}
                    </div>
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>
      )}
    </div>
  );
}
