// Repository interfaces — define the contracts the infrastructure layer must fulfil.
// Only domain entities are referenced here; no framework imports.

import type {
  AppUsageStat,
  CurrentStatus,
  DailySummary,
  Interval,
  Session,
  Settings,
  TodayStats,
} from './entities';

export interface ITrackingRepository {
  getCurrentStatus(): Promise<CurrentStatus>;
  getTodayStats(): Promise<TodayStats>;
}

export interface ISessionRepository {
  getSessionForDate(date: string): Promise<Session | null>;
  getIntervalsForSession(sessionId: string): Promise<Interval[]>;
}

export interface IHistoryRepository {
  getHistory(days: number): Promise<DailySummary[]>;
}

export interface IAppUsageRepository {
  getAppUsage(date: string): Promise<AppUsageStat[]>;
  /** Returns a native app icon data URI, or null if unavailable. */
  getAppIcon(appName: string): Promise<string | null>;
  /** Clears the backend icon cache. */
  clearIconCache(): Promise<void>;
}

export interface ISettingsRepository {
  getSettings(): Promise<Settings>;
  setSettings(settings: Settings): Promise<void>;
  clearAllData(): Promise<void>;
}
