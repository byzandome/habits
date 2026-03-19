import { invoke } from '@tauri-apps/api/core';

import type { Settings } from '../../domain/entities';
import type { ISettingsRepository } from '../../domain/repositories';

export class TauriSettingsRepository implements ISettingsRepository {
  getSettings(): Promise<Settings> {
    return invoke<Settings>('get_settings');
  }

  setSettings(settings: Settings): Promise<void> {
    return invoke<void>('set_settings', { ...settings });
  }

  clearAllData(): Promise<void> {
    return invoke<void>('clear_all_data');
  }
}
