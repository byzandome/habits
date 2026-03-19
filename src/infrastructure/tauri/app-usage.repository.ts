import { invoke } from '@tauri-apps/api/core';

import type { AppUsageStat } from '../../domain/entities';
import type { IAppUsageRepository } from '../../domain/repositories';

export class TauriAppUsageRepository implements IAppUsageRepository {
  getAppUsage(date: string): Promise<AppUsageStat[]> {
    return invoke<AppUsageStat[]>('get_app_usages', { date });
  }

  async getAppIcon(appName: string): Promise<string | null> {
    try {
      const dataUri = await invoke<string>('get_app_icon', { appName });
      return dataUri || null;
    } catch {
      return null;
    }
  }

  async clearIconCache(): Promise<void> {
    await invoke<void>('clear_icon_cache');
  }
}
