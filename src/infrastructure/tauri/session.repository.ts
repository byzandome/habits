import { invoke } from '@tauri-apps/api/core';

import type { Interval, Session } from '../../domain/entities';
import type { ISessionRepository } from '../../domain/repositories';

export class TauriSessionRepository implements ISessionRepository {
  getSessionForDate(date: string): Promise<Session | null> {
    return invoke<Session | null>('get_session_for_date', { date });
  }

  getIntervalsForSession(sessionId: string): Promise<Interval[]> {
    return invoke<Interval[]>('get_intervals_for_session', { session_id: sessionId });
  }
}
