use super::entities::{App, AppUsageStat, Domain, DomainHistory};

// ── Settings ───────────────────────────────────────────────────────────────────────

pub trait SettingsRepository: Send + Sync {
    fn get_setting(&self, key: &str) -> Result<Option<String>, String>;
    fn set_setting(&self, key: &str, value: &str) -> Result<(), String>;
}

// ── Apps ──────────────────────────────────────────────────────────────────────────

pub trait AppRepository: Send + Sync {
    fn upsert_app(&self, name: &str, path: &str) -> Result<App, String>;
    fn list_apps(&self) -> Result<Vec<App>, String>;
    fn find_app_by_name(&self, name: &str) -> Result<Option<App>, String>;
    fn update_app_color(&self, id: &str, color: Option<&str>) -> Result<(), String>;
    fn reset_all_colors(&self) -> Result<(), String>;
}

// ── App usages ──────────────────────────────────────────────────────────────────

pub trait AppUsageRepository: Send + Sync {
    fn begin_usage(&self, app_id: &str, start_at: &str) -> Result<String, String>;
    fn end_usage(&self, id: &str, end_at: &str, duration_secs: i64) -> Result<(), String>;
    // fn list_usages(&self, date: Option<&str>) -> Result<Vec<AppUsage>, String>;
    fn list_usage_stats(&self, date: Option<&str>) -> Result<Vec<AppUsageStat>, String>;
}

// ── Domains (write-side reserved for browser-extension integration) ────────────

#[allow(dead_code)]
pub trait DomainRepository: Send + Sync {
    fn upsert_domain(&self, url: &str, name: Option<&str>) -> Result<Domain, String>;
    fn list_domains(&self) -> Result<Vec<Domain>, String>;
}

// ── Domain history (write-side reserved for browser-extension integration) ─────

#[allow(dead_code)]
pub trait DomainHistoryRepository: Send + Sync {
    fn begin_visit(&self, domain_id: &str, url: &str, start_at: &str) -> Result<String, String>;
    fn end_visit(&self, id: &str, end_at: &str, duration_secs: i64) -> Result<(), String>;
    fn list_history(&self, date: Option<&str>) -> Result<Vec<DomainHistory>, String>;
}
