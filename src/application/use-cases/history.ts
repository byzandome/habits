import type { IHistoryRepository } from '../../domain/repositories';

export function createHistoryUseCases(repo: IHistoryRepository) {
  return {
    getHistory: (days: number) => repo.getHistory(days),
  };
}

export type HistoryUseCases = ReturnType<typeof createHistoryUseCases>;
