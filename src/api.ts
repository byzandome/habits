import { invoke } from '@tauri-apps/api/core';
import type { CurrentStatus, TodayStats, Session, DailySummary, Settings, AppUsageStat } from './types';

export const api = {
  getCurrentStatus: () =>
    invoke<CurrentStatus>('get_current_status'),

  getTodayStats: () =>
    invoke<TodayStats>('get_today_stats'),

  getSessionsForDate: (date: string) =>
    invoke<Session[]>('get_sessions_for_date', { date }),

  getHistory: (days = 7) =>
    invoke<DailySummary[]>('get_history', { days }),

  getSettings: () =>
    invoke<Settings>('get_settings'),

  setSettings: (settings: { idle_threshold_mins: number; autostart: boolean }) =>
    invoke<void>('set_settings', settings),

  getAppUsage: (date: string) =>
    invoke<AppUsageStat[]>('get_app_usage', { date }),
};
