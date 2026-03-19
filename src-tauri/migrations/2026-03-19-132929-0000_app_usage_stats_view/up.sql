CREATE VIEW app_usage_stats AS
SELECT
    a.id                                AS app_id,
    a.name                              AS app_name,
    a.path                              AS app_path,
    a.color                             AS app_color,
    DATE(au.start_at)                   AS usage_date,
    COUNT(au.id)                        AS session_count,
    COALESCE(SUM(au.duration_secs), 0)  AS total_duration_secs,
    MIN(au.start_at)                    AS first_usage_at,
    MAX(au.end_at)                      AS last_usage_at
FROM app_usages au
JOIN apps a ON a.id = au.app_id
WHERE au.duration_secs IS NOT NULL
GROUP BY a.id, DATE(au.start_at)
ORDER BY usage_date DESC, total_duration_secs DESC;
