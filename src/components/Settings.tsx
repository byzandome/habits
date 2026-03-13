import { useEffect, useState } from 'react';
import { api } from '../api';
import type { Settings } from '../types';

export function Settings() {
  const [settings, setSettings] = useState<Settings>({ idle_threshold_mins: 5, autostart: false });
  const [saved, setSaved] = useState(false);
  const [loading, setLoading] = useState(true);

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

  if (loading) return <div className="page"><div className="empty-state">Loading…</div></div>;

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
                  setSettings({ ...settings, idle_threshold_mins: Math.max(1, Math.min(60, Number(e.target.value))) })
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
              onChange={(e) => setSettings({ ...settings, idle_threshold_mins: Number(e.target.value) })}
            />
            <div style={{ display: 'flex', justifyContent: 'space-between', color: '#475569', fontSize: 11, marginTop: 4 }}>
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
              <div className="settings-desc">
                Automatically launch Habits when you log in.
              </div>
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
        <button
          className={`save-btn ${saved ? 'save-btn--saved' : ''}`}
          onClick={handleSave}
        >
          {saved ? '✓ Saved!' : 'Save Settings'}
        </button>
      </div>
    </div>
  );
}
