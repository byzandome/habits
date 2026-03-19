import type { IAppUsageRepository } from '../../domain/repositories';

export function createAppUsageUseCases(repo: IAppUsageRepository) {
  return {
    getAppUsage: (date: string) => repo.getAppUsage(date),
    /** Returns native icon data URI or null. Fallback chain handled by the hook. */
    getAppIcon: (appName: string) => repo.getAppIcon(appName),
    clearIconCache: () => repo.clearIconCache(),
  };
}

export type AppUsageUseCases = ReturnType<typeof createAppUsageUseCases>;
