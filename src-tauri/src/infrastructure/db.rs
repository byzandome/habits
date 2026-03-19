use std::sync::{Arc, Mutex};

use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use uuid::Uuid;

use crate::{
    domain::{
        entities::{App, AppUsageStat, Domain, DomainHistory},
        ports::{
            AppRepository, AppUsageRepository, DomainHistoryRepository, DomainRepository,
            SettingsRepository,
        },
    },
    schema::{app_usages, apps, domain_history, domains, settings},
};

use super::models::{NewApp, NewAppUsage, NewDomainHistory, NewDomain, NewSetting};

pub struct SqliteDb {
    conn: Arc<Mutex<SqliteConnection>>,
}

impl SqliteDb {
    pub fn new(conn: SqliteConnection) -> Self {
        Self { conn: Arc::new(Mutex::new(conn)) }
    }
}

impl SettingsRepository for SqliteDb {
    fn get_setting(&self, key: &str) -> Result<Option<String>, String> {
        let conn = &mut *self.conn.lock().unwrap();
        settings::table
            .filter(settings::key.eq(key))
            .select(settings::value)
            .first::<Option<String>>(conn)
            .optional()
            .map(|opt| opt.flatten())
            .map_err(|e| e.to_string())
    }

    fn set_setting(&self, key: &str, value: &str) -> Result<(), String> {
        let conn = &mut *self.conn.lock().unwrap();
        diesel::replace_into(settings::table)
            .values(NewSetting { key, value: Some(value) })
            .execute(conn)
            .map(|_| ())
            .map_err(|e| e.to_string())
    }
}

impl AppRepository for SqliteDb {
    fn upsert_app(&self, name: &str, path: &str) -> Result<App, String> {
        let conn = &mut *self.conn.lock().unwrap();
        if let Some(row) = apps::table
            .filter(apps::path.eq(path))
            .select((apps::id, apps::name, apps::path, apps::color))
            .first::<(String, String, String, Option<String>)>(conn)
            .optional()
            .map_err(|e| e.to_string())?
        {
            return Ok(App { id: row.0, name: row.1, path: row.2, color: row.3 });
        }
        let id = Uuid::new_v4().to_string();
        diesel::insert_into(apps::table)
            .values(NewApp { id: &id, name, path })
            .execute(conn)
            .map_err(|e| e.to_string())?;
        Ok(App { id, name: name.to_string(), path: path.to_string(), color: None })
    }

    fn list_apps(&self) -> Result<Vec<App>, String> {
        let conn = &mut *self.conn.lock().unwrap();
        apps::table
            .select((apps::id, apps::name, apps::path, apps::color))
            .order(apps::name.asc())
            .load::<(String, String, String, Option<String>)>(conn)
            .map(|rows| rows.into_iter().map(|(id, name, path, color)| App { id, name, path, color }).collect())
            .map_err(|e| e.to_string())
    }

    fn find_app_by_name(&self, name: &str) -> Result<Option<App>, String> {
        let conn = &mut *self.conn.lock().unwrap();
        apps::table
            .filter(apps::name.eq(name))
            .select((apps::id, apps::name, apps::path, apps::color))
            .first::<(String, String, String, Option<String>)>(conn)
            .optional()
            .map(|opt| opt.map(|(id, name, path, color)| App { id, name, path, color }))
            .map_err(|e| e.to_string())
    }

    fn update_app_color(&self, id: &str, color: Option<&str>) -> Result<(), String> {
        let conn = &mut *self.conn.lock().unwrap();
        diesel::update(apps::table.filter(apps::id.eq(id)))
            .set(apps::color.eq(color))
            .execute(conn)
            .map(|_| ())
            .map_err(|e| e.to_string())
    }

    fn reset_all_colors(&self) -> Result<(), String> {
        let conn = &mut *self.conn.lock().unwrap();
        diesel::update(apps::table)
            .set(apps::color.eq(None::<String>))
            .execute(conn)
            .map(|_| ())
            .map_err(|e| e.to_string())
    }
}

impl AppUsageRepository for SqliteDb {
    fn begin_usage(&self, app_id: &str, start_at: &str) -> Result<String, String> {
        let id = Uuid::new_v4().to_string();
        let conn = &mut *self.conn.lock().unwrap();
        diesel::insert_into(app_usages::table)
            .values(NewAppUsage { id: &id, start_at, app_id: Some(app_id) })
            .execute(conn)
            .map(|_| id)
            .map_err(|e| e.to_string())
    }

    fn end_usage(&self, id: &str, end_at: &str, duration_secs: i64) -> Result<(), String> {
        let conn = &mut *self.conn.lock().unwrap();
        diesel::update(app_usages::table.filter(app_usages::id.eq(id)))
            .set((
                app_usages::end_at.eq(end_at),
                app_usages::duration_secs.eq(duration_secs),
            ))
            .execute(conn)
            .map(|_| ())
            .map_err(|e| e.to_string())
    }

    fn list_usage_stats(&self, date: Option<&str>) -> Result<Vec<AppUsageStat>, String> {
        let conn = &mut *self.conn.lock().unwrap();
        if let Some(d) = date {
            diesel::sql_query(
                "SELECT app_id AS id, app_name AS app_id, \
                 app_name, \
                 total_duration_secs AS duration_secs, \
                 first_usage_at AS start_at, \
                 COALESCE(last_usage_at, '') AS end_at \
                 FROM app_usage_stats WHERE usage_date = ? \
                 ORDER BY total_duration_secs DESC"
            )
            .bind::<diesel::sql_types::Text, _>(d.to_string())
            .load::<AppUsageStat>(conn)
            .map_err(|e| e.to_string())
        } else {
            diesel::sql_query(
                "SELECT app_id AS id, app_name AS app_id, \
                 app_name, \
                 total_duration_secs AS duration_secs, \
                 first_usage_at AS start_at, \
                 COALESCE(last_usage_at, '') AS end_at \
                 FROM app_usage_stats \
                 ORDER BY total_duration_secs DESC"
            )
            .load::<AppUsageStat>(conn)
            .map_err(|e| e.to_string())
        }
    }
}

impl DomainRepository for SqliteDb {
    fn upsert_domain(&self, url: &str, name: Option<&str>) -> Result<Domain, String> {
        let conn = &mut *self.conn.lock().unwrap();
        if let Some(row) = domains::table
            .filter(domains::url.eq(url))
            .select((domains::id, domains::url, domains::name))
            .first::<(String, String, Option<String>)>(conn)
            .optional()
            .map_err(|e| e.to_string())?
        {
            return Ok(Domain { id: row.0, url: row.1, name: row.2 });
        }
        let id = Uuid::new_v4().to_string();
        diesel::insert_into(domains::table)
            .values(NewDomain { id: &id, url, name })
            .execute(conn)
            .map_err(|e| e.to_string())?;
        Ok(Domain { id, url: url.to_string(), name: name.map(str::to_string) })
    }

    fn list_domains(&self) -> Result<Vec<Domain>, String> {
        let conn = &mut *self.conn.lock().unwrap();
        domains::table
            .select((domains::id, domains::url, domains::name))
            .order(domains::url.asc())
            .load::<(String, String, Option<String>)>(conn)
            .map(|rows| rows.into_iter().map(|(id, url, name)| Domain { id, url, name }).collect())
            .map_err(|e| e.to_string())
    }
}

impl DomainHistoryRepository for SqliteDb {
    fn begin_visit(&self, domain_id: &str, url: &str, start_at: &str) -> Result<String, String> {
        let id = Uuid::new_v4().to_string();
        let conn = &mut *self.conn.lock().unwrap();
        diesel::insert_into(domain_history::table)
            .values(NewDomainHistory { id: &id, domain_id, url, start_at })
            .execute(conn)
            .map(|_| id)
            .map_err(|e| e.to_string())
    }

    fn end_visit(&self, id: &str, end_at: &str, duration_secs: i64) -> Result<(), String> {
        let conn = &mut *self.conn.lock().unwrap();
        diesel::update(domain_history::table.filter(domain_history::id.eq(id)))
            .set((
                domain_history::end_at.eq(end_at),
                domain_history::duration_secs.eq(duration_secs),
            ))
            .execute(conn)
            .map(|_| ())
            .map_err(|e| e.to_string())
    }

    fn list_history(&self, date: Option<&str>) -> Result<Vec<DomainHistory>, String> {
        let conn = &mut *self.conn.lock().unwrap();
        let mut query = domain_history::table
            .select((
                domain_history::id,
                domain_history::domain_id,
                domain_history::url,
                domain_history::start_at,
                domain_history::end_at,
                domain_history::duration_secs,
            ))
            .order(domain_history::start_at.desc())
            .into_boxed();
        if let Some(d) = date {
            query = query.filter(domain_history::start_at.like(format!("{d}%")));
        }
        query
            .load::<(String, String, String, String, Option<String>, Option<i64>)>(conn)
            .map(|rows| {
                rows.into_iter()
                    .map(|(id, domain_id, url, start_at, end_at, duration_secs)| DomainHistory {
                        id, domain_id, url, start_at, end_at, duration_secs,
                    })
                    .collect()
            })
            .map_err(|e| e.to_string())
    }
}