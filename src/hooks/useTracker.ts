import { useState, useEffect, useRef } from 'react';
import { api } from '../api';
import type { CurrentStatus, TodayStats } from '../types';

export interface TrackerState {
  status: 'productive' | 'idle';
  sessionDurationSecs: number;
  productiveSecs: number;
  idleSecs: number;
}

const DEFAULT: TrackerState = {
  status: 'productive',
  sessionDurationSecs: 0,
  productiveSecs: 0,
  idleSecs: 0,
};

/** Polls the Rust backend every 10 s and keeps a locally-ticking second counter. */
export function useTracker(): TrackerState {
  const [server, setServer] = useState<{ status: CurrentStatus; today: TodayStats } | null>(null);
  const [displaySecs, setDisplaySecs] = useState(0);
  const tickRef = useRef<ReturnType<typeof setInterval> | null>(null);

  // ── Poll backend every 10 seconds ────────────────────────────────────────
  useEffect(() => {
    const poll = async () => {
      try {
        const [status, today] = await Promise.all([
          api.getCurrentStatus(),
          api.getTodayStats(),
        ]);
        setServer({ status, today });
        setDisplaySecs(status.session_duration_secs);
      } catch {
        // backend may not be ready yet on first render
      }
    };

    poll();
    const id = setInterval(poll, 10_000);
    return () => clearInterval(id);
  }, []);

  // ── Tick display counter every second ────────────────────────────────────
  useEffect(() => {
    if (tickRef.current) clearInterval(tickRef.current);
    tickRef.current = setInterval(() => setDisplaySecs((s) => s + 1), 1_000);
    return () => {
      if (tickRef.current) clearInterval(tickRef.current);
    };
  }, [server?.status.status]); // restart counter when status flips

  if (!server) return DEFAULT;

  return {
    status: server.status.status,
    sessionDurationSecs: displaySecs,
    productiveSecs: server.today.productive_secs,
    idleSecs: server.today.idle_secs,
  };
}
