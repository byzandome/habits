import { invoke } from '@tauri-apps/api/core';

import type { DailySummary } from '../../domain/entities';
import type { IHistoryRepository } from '../../domain/repositories';

export class TauriHistoryRepository implements IHistoryRepository {
  getHistory(days: number): Promise<DailySummary[]> {
    return invoke<DailySummary[]>('get_history', { days });
  }
}
