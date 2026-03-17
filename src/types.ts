// Mirrors Rust response types from commands.rs

export interface CurrentStatus {
  status: 'productive' | 'idle' | 'locked';
  session_duration_secs: number;
}

export interface TodayStats {
  productive_secs: number;
  idle_secs: number;
  locked_secs: number;
}

export interface Session {
  id: string;
  date: string;         // "YYYY-MM-DD" local date
  start_time: string;  // ISO 8601 UTC
  end_time: string;    // ISO 8601 UTC, empty string = in-progress
  active_secs: number;
  idle_secs: number;
  locked_secs: number;
  unknown_secs: number;
}

export interface DailySummary {
  date: string; // "YYYY-MM-DD" local
  productive_secs: number;
  idle_secs: number;
  locked_secs: number;
}

export interface Settings {
  idle_threshold_mins: number;
  autostart: boolean;
}

export interface AppUsageStat {
  app_name: string;
  duration_secs: number;
  exe_path: string;
  pct_of_day: number;
}

export interface Interval {
  id: string;
  session_id: string;
  app_usage_id: string | null;
  start_time: string;
  end_time: string | null;
  duration_secs: number;
  type: 'active' | 'idle' | 'locked' | 'unknown';
}
