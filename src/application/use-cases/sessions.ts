import type { ISessionRepository } from '../../domain/repositories';

export function createSessionUseCases(repo: ISessionRepository) {
  return {
    getSessionForDate: (date: string) => repo.getSessionForDate(date),
    getIntervalsForSession: (sessionId: string) => repo.getIntervalsForSession(sessionId),
  };
}

export type SessionUseCases = ReturnType<typeof createSessionUseCases>;
