interface Props {
  status: 'productive' | 'idle';
  sessionDurationSecs: number;
}

function formatDuration(secs: number): string {
  const h = Math.floor(secs / 3600);
  const m = Math.floor((secs % 3600) / 60);
  const s = secs % 60;
  if (h > 0) return `${h}h ${String(m).padStart(2, '0')}m ${String(s).padStart(2, '0')}s`;
  if (m > 0) return `${m}m ${String(s).padStart(2, '0')}s`;
  return `${s}s`;
}

export function StatusBadge({ status, sessionDurationSecs }: Props) {
  const isProductive = status === 'productive';

  return (
    <div className={`status-badge ${isProductive ? 'status-badge--productive' : 'status-badge--idle'}`}>
      <span className={`badge-dot ${isProductive ? 'badge-dot--active' : ''}`} />
      <div>
        <span className="badge-label">{isProductive ? 'Productive' : 'Idle'}</span>
        <span className="badge-timer">{formatDuration(sessionDurationSecs)}</span>
      </div>
    </div>
  );
}
