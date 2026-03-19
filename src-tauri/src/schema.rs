// @generated — mirrors migrations/2026-03-17-000000_initial_schema/up.sql
// Regenerate with: diesel print-schema  (run from src-tauri/)

diesel::table! {
    settings (key) {
        key -> Text,
        value -> Nullable<Text>,
    }
}

diesel::table! {
    apps (id) {
        id -> Text,
        name -> Text,
        path -> Text,
        color -> Nullable<Text>,
    }
}

diesel::table! {
    app_usages (id) {
        id -> Text,
        start_at -> Text,
        duration_secs -> Nullable<BigInt>,
        end_at -> Nullable<Text>,
        app_id -> Nullable<Text>,
    }
}

diesel::table! {
    domains (id) {
        id -> Text,
        url -> Text,
        name -> Nullable<Text>,
    }
}

diesel::table! {
    domain_history (id) {
        id -> Text,
        domain_id -> Text,
        url -> Text,
        start_at -> Text,
        end_at -> Nullable<Text>,
        duration_secs -> Nullable<BigInt>,
    }
}

diesel::joinable!(app_usages -> apps (app_id));
diesel::joinable!(domain_history -> domains (domain_id));

diesel::allow_tables_to_appear_in_same_query!(settings, apps, app_usages, domains, domain_history,);
