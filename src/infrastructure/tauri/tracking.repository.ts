import { invoke } from '@tauri-apps/api/core';

import type { CurrentStatus, TodayStats } from '../../domain/entities';
import type { ITrackingRepository } from '../../domain/repositories';

export class TauriTrackingRepository implements ITrackingRepository {
  getCurrentStatus(): Promise<CurrentStatus> {
    return invoke<CurrentStatus>('get_current_status');
  }

  getTodayStats(): Promise<TodayStats> {
    return invoke<TodayStats>('get_today_stats');
  }
}
