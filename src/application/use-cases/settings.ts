import type { Settings } from '../../domain/entities';
import type { ISettingsRepository } from '../../domain/repositories';

export function createSettingsUseCases(repo: ISettingsRepository) {
  return {
    getSettings: () => repo.getSettings(),
    setSettings: (settings: Settings) => repo.setSettings(settings),
    clearAllData: () => repo.clearAllData(),
  };
}

export type SettingsUseCases = ReturnType<typeof createSettingsUseCases>;
