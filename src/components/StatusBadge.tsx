interface Props {
  status: 'productive' | 'idle' | 'locked';
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
  const isLocked = status === 'locked';

  const badgeClass = isProductive
    ? 'status-badge--productive'
    : isLocked
      ? 'status-badge--locked'
      : 'status-badge--idle';

  const label = isProductive ? 'Productive' : isLocked ? 'Locked' : 'Idle';

  return (
    <div className={`status-badge ${badgeClass}`}>
      <span
        className={`badge-dot ${isProductive ? 'badge-dot--active' : isLocked ? 'badge-dot--locked' : ''}`}
      />
      <div>
        <span className="badge-label">{label}</span>
        <span className="badge-timer">{formatDuration(sessionDurationSecs)}</span>
      </div>
    </div>
  );
}
