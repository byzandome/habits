import type { ITrackingRepository } from '../../domain/repositories';

export function createTrackingUseCases(repo: ITrackingRepository) {
  return {
    getCurrentStatus: () => repo.getCurrentStatus(),
    getTodayStats: () => repo.getTodayStats(),
  };
}

export type TrackingUseCases = ReturnType<typeof createTrackingUseCases>;
