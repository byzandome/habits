use diesel::prelude::*;

use crate::schema::{app_usages, apps, domain_history, domains, settings};

// ── settings ─────────────────────────────────────────────────────────────────────

#[derive(Insertable)]
#[diesel(table_name = settings)]
pub struct NewSetting<'a> {
    pub key: &'a str,
    pub value: Option<&'a str>,
}

// ── apps ──────────────────────────────────────────────────────────────────────────

#[derive(Insertable)]
#[diesel(table_name = apps)]
pub struct NewApp<'a> {
    pub id: &'a str,
    pub name: &'a str,
    pub path: &'a str,
}

// ── app_usages ───────────────────────────────────────────────────────────────────

#[derive(Insertable)]
#[diesel(table_name = app_usages)]
pub struct NewAppUsage<'a> {
    pub id: &'a str,
    pub start_at: &'a str,
    pub app_id: Option<&'a str>,
}

// ── domains ──────────────────────────────────────────────────────────────────────

#[derive(Insertable)]
#[diesel(table_name = domains)]
pub struct NewDomain<'a> {
    pub id: &'a str,
    pub url: &'a str,
    pub name: Option<&'a str>,
}

// ── domain_history ───────────────────────────────────────────────────────────────

#[derive(Insertable)]
#[diesel(table_name = domain_history)]
pub struct NewDomainHistory<'a> {
    pub id: &'a str,
    pub domain_id: &'a str,
    pub url: &'a str,
    pub start_at: &'a str,
}
