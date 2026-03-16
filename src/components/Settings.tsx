import type { Settings } from '../types';

import { useEffect, useState } from 'react';

import { api } from '../api';
// Module-level icon cache exported from AppUsage — we clear it here directly.
// Import the Map reference so we can wipe it without re-mounting the component.
import { iconCache } from './AppUsage';

export function Settings() {
  const [settings, setSettings] = useState<Settings>({ idle_threshold_mins: 5, autostart: false });
  const [saved, setSaved] = useState(false);
  const [loading, setLoading] = useState(true);
  const [clearing, setClearing] = useState(false);
  const [cleared, setCleared] = useState(false);
  const [confirmDelete, setConfirmDelete] = useState(false);
  const [deleting, setDeleting] = useState(false);
  const [deleted, setDeleted] = useState(false);

  const handleClearCache = async () => {
    setClearing(true);
    try {
      await api.clearIconCache();
      iconCache.clear();
      setCleared(true);
      setTimeout(() => setCleared(false), 2500);
    } finally {
      setClearing(false);
    }
  };

  const handleDeleteData = async () => {
    if (!confirmDelete) {
      setConfirmDelete(true);
      return;
    }
    setDeleting(true);
    try {
      await api.clearAllData();
      iconCache.clear();
      setDeleted(true);
      setConfirmDelete(false);
      setTimeout(() => setDeleted(false), 3000);
    } finally {
      setDeleting(false);
    }
  };

  useEffect(() => {
    api.getSettings().then((s) => {
      setSettings(s);
      setLoading(false);
    });
  }, []);

  const handleSave = async () => {
    await api.setSettings(settings);
    setSaved(true);
    setTimeout(() => setSaved(false), 2000);
  };

  if (loading)
    return (
      <div className="page">
        <div className="empty-state">Loading…</div>
      </div>
    );

  return (
    <div className="page">
      <div className="page-header">
        <div>
          <h1 className="page-title">Settings</h1>
          <p className="page-subtitle">Configure tracking behaviour</p>
        </div>
      </div>

      <div style={{ display: 'flex', flexDirection: 'column', gap: 16, maxWidth: 520 }}>
        {/* Idle threshold */}
        <div className="card settings-card">
          <div className="settings-row">
            <div>
              <div className="settings-label">Idle Threshold</div>
              <div className="settings-desc">
                Minutes of inactivity before switching to "Idle" state.
              </div>
            </div>
            <div style={{ display: 'flex', alignItems: 'center', gap: 10 }}>
              <input
                type="number"
                className="number-input"
                min={1}
                max={60}
                value={settings.idle_threshold_mins}
                onChange={(e) =>
                  setSettings({
                    ...settings,
                    idle_threshold_mins: Math.max(1, Math.min(60, Number(e.target.value))),
                  })
                }
              />
              <span style={{ color: '#64748B', fontSize: 13 }}>min</span>
            </div>
          </div>

          {/* Slider */}
          <div style={{ marginTop: 16 }}>
            <input
              type="range"
              className="range-input"
              min={1}
              max={30}
              value={settings.idle_threshold_mins}
              onChange={(e) =>
                setSettings({ ...settings, idle_threshold_mins: Number(e.target.value) })
              }
            />
            <div
              style={{
                display: 'flex',
                justifyContent: 'space-between',
                color: '#475569',
                fontSize: 11,
                marginTop: 4,
              }}
            >
              <span>1 min</span>
              <span>30 min</span>
            </div>
          </div>
        </div>

        {/* Autostart */}
        <div className="card settings-card">
          <div className="settings-row">
            <div>
              <div className="settings-label">Start with Windows</div>
              <div className="settings-desc">Automatically launch Habits when you log in.</div>
            </div>
            <button
              className={`toggle-btn ${settings.autostart ? 'toggle-btn--on' : ''}`}
              onClick={() => setSettings({ ...settings, autostart: !settings.autostart })}
              aria-label="Toggle autostart"
            >
              <span className="toggle-thumb" />
            </button>
          </div>
        </div>

        {/* Save */}
        <button className={`save-btn ${saved ? 'save-btn--saved' : ''}`} onClick={handleSave}>
          {saved ? '✓ Saved!' : 'Save Settings'}
        </button>

        {/* Clear cache */}
        <div className="card settings-card" style={{ marginTop: 8 }}>
          <div className="settings-row">
            <div>
              <div className="settings-label">Clear Icon Cache</div>
              <div className="settings-desc">
                Forces all app icons to be re-fetched on the next visit. Useful after updating an
                app or if icons appear broken.
              </div>
            </div>
            <button
              className={`danger-btn ${cleared ? 'danger-btn--done' : ''}`}
              onClick={handleClearCache}
              disabled={clearing}
            >
              {cleared ? '✓ Cleared' : clearing ? 'Clearing…' : 'Clear Cache'}
            </button>
          </div>
        </div>

        {/* Delete all data */}
        <div
          className="card settings-card"
          style={{ borderColor: confirmDelete ? 'rgba(239,68,68,0.40)' : undefined }}
        >
          <div className="settings-row">
            <div>
              <div className="settings-label" style={{ color: '#F87171' }}>
                Delete All Data
              </div>
              <div className="settings-desc">
                Permanently removes all sessions, app-usage history and icon cache. Settings are
                kept. This cannot be undone.
              </div>
            </div>
            <div style={{ display: 'flex', gap: 8, flexShrink: 0 }}>
              {confirmDelete && (
                <button
                  className="danger-btn"
                  style={{
                    borderColor: 'rgba(100,116,139,0.30)',
                    background: 'rgba(100,116,139,0.10)',
                    color: '#94A3B8',
                  }}
                  onClick={() => setConfirmDelete(false)}
                  disabled={deleting}
                >
                  Cancel
                </button>
              )}
              <button
                className={`danger-btn ${deleted ? 'danger-btn--done' : ''}`}
                onClick={handleDeleteData}
                disabled={deleting}
              >
                {deleted
                  ? '✓ Deleted'
                  : deleting
                    ? 'Deleting…'
                    : confirmDelete
                      ? 'Confirm Delete'
                      : 'Delete All Data'}
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
