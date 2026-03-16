import { useState, useEffect, useRef } from 'react';
import { listen } from '@tauri-apps/api/event';
import { api } from '../api';
import type { CurrentStatus, TodayStats } from '../types';

export interface TrackerState {
  status: 'productive' | 'idle' | 'locked';
  sessionDurationSecs: number;
  productiveSecs: number;
  idleSecs: number;
  lockedSecs: number;
}

const DEFAULT: TrackerState = {
  status: 'productive',
  sessionDurationSecs: 0,
  productiveSecs: 0,
  idleSecs: 0,
  lockedSecs: 0,
};

/** Polls the Rust backend every 5 s and reacts instantly to Tauri events. */
export function useTracker(): TrackerState {
  const [server, setServer] = useState<{ status: CurrentStatus; today: TodayStats } | null>(null);
  const [displaySecs, setDisplaySecs] = useState(0);
  const tickRef = useRef<ReturnType<typeof setInterval> | null>(null);

  // ── Poll backend + subscribe to instant status events ────────────────────
  useEffect(() => {
    let unlistenFn: (() => void) | undefined;

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

    const setup = async () => {
      await poll();
      const id = setInterval(poll, 5_000);

      // The Rust tracker emits this event the moment status flips
      // (lock/unlock: near-instant via WTS; idle: within 5 s).
      unlistenFn = await listen<string>('tracker-status-changed', () => {
        poll();
      });

      return () => {
        clearInterval(id);
        unlistenFn?.();
      };
    };

    const cleanupPromise = setup();
    return () => {
      cleanupPromise.then((cleanup) => cleanup?.());
    };
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
    lockedSecs: server.today.locked_secs,
  };
}
