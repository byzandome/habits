export const STATUSES = {
    PRODUCTIVE: 'productive',
    IDLE: 'idle',
    LOCKED: 'locked',
    UNKNOWN: 'unknown',
} as const;

export type Status = typeof STATUSES[keyof typeof STATUSES];