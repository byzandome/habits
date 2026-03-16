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
  id: number;
  start_time: string; // ISO 8601 UTC
  end_time: string; // ISO 8601 UTC, empty string = in-progress
  active_secs: number;
  idle_secs: number;
  locked_secs: number;
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
}
