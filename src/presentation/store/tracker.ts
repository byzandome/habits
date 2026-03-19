import { listen } from '@tauri-apps/api/event';
import { create } from 'zustand';

import { trackingUseCases } from '../../infrastructure/container';

export interface TrackerState {
  status: 'productive' | 'idle' | 'locked';
  sessionDurationSecs: number;
  productiveSecs: number;
  idleSecs: number;
  lockedSecs: number;
}

interface TrackerStore extends TrackerState {
  /** Call once on mount; returns a cleanup function. */
  init: () => Promise<() => void>;
}

export const useTrackerStore = create<TrackerStore>((set, get) => ({
  status: 'productive',
  sessionDurationSecs: 0,
  productiveSecs: 0,
  idleSecs: 0,
  lockedSecs: 0,

  init: async () => {
    let tickId: ReturnType<typeof setInterval> | null = null;

    const startTick = () => {
      if (tickId) clearInterval(tickId);
      tickId = setInterval(
        () => set((s) => ({ sessionDurationSecs: s.sessionDurationSecs + 1 })),
        1_000,
      );
    };

    const poll = async () => {
      try {
        const [status, today] = await Promise.all([
          trackingUseCases.getCurrentStatus(),
          trackingUseCases.getTodayStats(),
        ] as const);
        const prevStatus = get().status;
        set({
          status: status.status,
          sessionDurationSecs: status.session_duration_secs,
          productiveSecs: today.productive_secs,
          idleSecs: today.idle_secs,
          lockedSecs: today.locked_secs,
        });
        if (status.status !== prevStatus) startTick();
      } catch {
        // backend may not be ready yet on first render
      }
    };

    await poll();
    startTick();
    const pollId = setInterval(poll, 5_000);
    const unlisten = await listen<string>('tracker-status-changed', poll);

    return () => {
      clearInterval(pollId);
      if (tickId) clearInterval(tickId);
      unlisten();
    };
  },
}));
