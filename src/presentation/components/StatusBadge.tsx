import { STATUSES, type Status } from '@/shared/constants/Status';
import { formatCurrentDistanceDateToNow } from '@/shared/utils/date';
import { cn } from '@/shared/utils/theme';
import { useMemo } from 'react';

type StatusBadgeProps = {
  status: Status;
  startSessionDate: Date;
};

export function StatusBadge({ status }: StatusBadgeProps) {
  const isProductive = useMemo(() => status === STATUSES.PRODUCTIVE, [status]);
  const isLocked = useMemo(() => status === STATUSES.LOCKED, [status]);
  const isIdle = useMemo(() => status === STATUSES.IDLE, [status]);

  const label = useMemo(() => {
    if (isProductive) return 'Productive';
    if (isLocked) return 'Locked';
    if (isIdle) return 'Idle';
    return 'Unknown';
  }, [isProductive, isLocked, isIdle]);

  return (
    <div
      className={cn('border rounded-xl gap-2.5 py-2 pr-4 pl-3 flex items-center select-none', {
        'bg-productive/20 border-productive/50': isProductive,
        'bg-locked/20 border-locked/50': isLocked,
        'bg-idle/20 border-idle/50': isIdle,
      })}
    >
      <span
        className={cn(
          'w-2 h-2 rounded-full bg-accent shrink-0 animate-caret-blink fade-in-20',
          {
            'bg-productive': isProductive,
            'bg-locked': isLocked,
            'bg-idle': isIdle,
          },
        )}
      />
      <div className="flex items-center gap-x-2">
        <span className="text-sm font-semibold text-foreground">{label}</span>
        <span className="text-xs tabular-nums text-gray-500">
          {formatCurrentDistanceDateToNow(new Date(2026, 2, 17, 20, 19))}
        </span>
      </div>
    </div>
  );
}
