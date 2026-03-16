import type { AppUsageStat } from '../types';

import { invoke } from '@tauri-apps/api/core';
import { format } from 'date-fns';
import { useCallback, useEffect, useRef, useState } from 'react';

import { api } from '../api';
import { getDisplayName, getFallbackDomain } from '../appMeta';

// ── Module-level icon cache (survives re-renders, cleared on page refresh) ───
export const iconCache = new Map<string, string | null>();
const iconPending = new Set<string>();

// ── Helpers ───────────────────────────────────────────────────────────────────

function formatHM(secs: number): string {
  const h = Math.floor(secs / 3600);
  const m = Math.floor((secs % 3600) / 60);
  if (h > 0) return `${h}h ${m}m`;
  if (m > 0) return `${m}m`;
  return `${secs}s`;
}

/** djb2 hash → one of 8 vibrant palette colours. */
function nameToColor(name: string): string {
  const palette = [
    '#3B82F6',
    '#8B5CF6',
    '#EC4899',
    '#F59E0B',
    '#10B981',
    '#06B6D4',
    '#EF4444',
    '#84CC16',
  ];
  let h = 0;
  for (let i = 0; i < name.length; i++) {
    h = (h << 5) - h + name.charCodeAt(i);
    h |= 0;
  }
  return palette[Math.abs(h) % palette.length];
}

function hexToRgb(hex: string): string {
  const r = parseInt(hex.slice(1, 3), 16);
  const g = parseInt(hex.slice(3, 5), 16);
  const b = parseInt(hex.slice(5, 7), 16);
  return `${r},${g},${b}`;
}

/** Draws `img` on an off-screen canvas and returns the dominant non-white,
 *  non-black colour as `"R,G,B"`, or `""` on any failure (including CORS). */
function extractDominantColor(img: HTMLImageElement): string {
  try {
    const S = 32;
    const canvas = document.createElement('canvas');
    canvas.width = S;
    canvas.height = S;
    const ctx = canvas.getContext('2d');
    if (!ctx) return '';
    ctx.drawImage(img, 0, 0, S, S);
    const { data } = ctx.getImageData(0, 0, S, S);
    const counts: Record<string, number> = {};
    for (let i = 0; i < data.length; i += 4) {
      const r = data[i],
        g = data[i + 1],
        b = data[i + 2],
        a = data[i + 3];
      if (a < 128) continue;
      if (r > 220 && g > 220 && b > 220) continue; // near-white
      if (r < 30 && g < 30 && b < 30) continue; // near-black
      const key = `${Math.round(r / 32) * 32},${Math.round(g / 32) * 32},${Math.round(b / 32) * 32}`;
      counts[key] = (counts[key] ?? 0) + 1;
    }
    let maxCount = 0,
      dominant = '';
    for (const [k, c] of Object.entries(counts)) {
      if (c > maxCount) {
        maxCount = c;
        dominant = k;
      }
    }
    return dominant;
  } catch {
    return '';
  }
}

// ── useAppIcon ────────────────────────────────────────────────────────────────

function useAppIcon(appName: string): string | null {
  const [icon, setIcon] = useState<string | null>(() => iconCache.get(appName) ?? null);

  useEffect(() => {
    if (iconCache.has(appName) || iconPending.has(appName)) return;
    iconPending.add(appName);

    invoke<string>('get_app_icon', { appName })
      .then((dataUri) => {
        iconPending.delete(appName);
        if (dataUri) {
          iconCache.set(appName, dataUri);
          setIcon(dataUri);
        } else {
          const domain = getFallbackDomain(appName);
          const url = domain ? `https://icons.duckduckgo.com/ip3/${domain}.ico` : null;
          iconCache.set(appName, url);
          setIcon(url);
        }
      })
      .catch(() => {
        iconPending.delete(appName);
        iconCache.set(appName, null);
      });
  }, [appName]);

  return icon;
}

// ── AppIconImg ────────────────────────────────────────────────────────────────

interface AppIconImgProps {
  appName: string;
  onColorReady: (appName: string, rgb: string) => void;
}

function AppIconImg({ appName, onColorReady }: AppIconImgProps) {
  const iconSrc = useAppIcon(appName);
  const [imgFailed, setImgFailed] = useState(false);
  const displayName = getDisplayName(appName);
  const avatarColor = nameToColor(appName);
  const showAvatar = iconSrc === null || imgFailed;

  // When no icon is available, seed the shadow with the avatar's palette colour
  useEffect(() => {
    if (showAvatar) {
      onColorReady(appName, hexToRgb(avatarColor));
    }
  }, [showAvatar, avatarColor, appName, onColorReady]);

  if (showAvatar) {
    return (
      <div
        style={{
          width: 28,
          height: 28,
          borderRadius: 8,
          background: avatarColor,
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          fontSize: 13,
          fontWeight: 700,
          color: '#fff',
          flexShrink: 0,
        }}
      >
        {displayName[0]?.toUpperCase() ?? '?'}
      </div>
    );
  }

  return (
    <img
      src={iconSrc!}
      width={28}
      height={28}
      style={{ borderRadius: 6, objectFit: 'contain', flexShrink: 0 }}
      alt={displayName}
      onLoad={(e) => {
        const rgb = extractDominantColor(e.currentTarget);
        if (rgb) onColorReady(appName, rgb);
      }}
      onError={() => setImgFailed(true)}
    />
  );
}

function RefreshIcon() {
  return (
    <svg
      width="15"
      height="15"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="2.5"
      strokeLinecap="round"
      strokeLinejoin="round"
    >
      <polyline points="23 4 23 10 17 10" />
      <polyline points="1 20 1 14 7 14" />
      <path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15" />
    </svg>
  );
}

export function AppUsage() {
  const todayStr = format(new Date(), 'yyyy-MM-dd');
  const [date, setDate] = useState(todayStr);
  const [stats, setStats] = useState<AppUsageStat[]>([]);
  const [loading, setLoading] = useState(true);
  const [refreshing, setRefreshing] = useState(false);
  const [cardColors, setCardColors] = useState<Record<string, string>>({});
  const intervalRef = useRef<ReturnType<typeof setInterval> | null>(null);

  const fetchData = useCallback((d: string, isRefresh = false) => {
    if (isRefresh) {
      setRefreshing(true);
    } else {
      setLoading(true);
    }
    api.getAppUsage(d).then((data) => {
      setStats(data);
      setLoading(false);
      setRefreshing(false);
    });
  }, []);

  useEffect(() => {
    fetchData(date);
  }, [date, fetchData]);

  // Auto-refresh every 5 min, only when viewing today
  useEffect(() => {
    if (date !== todayStr) return;
    intervalRef.current = setInterval(() => fetchData(date, true), 5 * 60 * 1000);
    return () => {
      if (intervalRef.current) clearInterval(intervalRef.current);
    };
  }, [date, todayStr, fetchData]);

  const handleColorReady = useCallback((appName: string, rgb: string) => {
    setCardColors((prev) => {
      if (prev[appName] === rgb) return prev;
      return { ...prev, [appName]: rgb };
    });
  }, []);

  const totalSecs = stats.reduce((sum, s) => sum + s.duration_secs, 0);
  const maxSecs = stats[0]?.duration_secs ?? 1;

  return (
    <div className="page">
      <div className="page-header">
        <div>
          <h1 className="page-title">App Usage</h1>
          <p className="page-subtitle">Productive time per application</p>
        </div>
        <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
          <button
            className={`refresh-btn${refreshing || loading ? ' refresh-btn--spinning' : ''}`}
            onClick={() => fetchData(date, true)}
            disabled={loading || refreshing}
            title="Refresh"
          >
            <RefreshIcon />
          </button>
          <input
            type="date"
            className="date-input"
            value={date}
            max={todayStr}
            onChange={(e) => setDate(e.target.value)}
          />
        </div>
      </div>

      {/* Summary card */}
      {!loading && stats.length > 0 && (
        <div
          className="card"
          style={{
            padding: '16px 20px',
            marginBottom: 12,
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'space-between',
          }}
        >
          <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
            <svg
              width="16"
              height="16"
              viewBox="0 0 24 24"
              fill="none"
              stroke="#22C55E"
              strokeWidth="2"
              strokeLinecap="round"
              strokeLinejoin="round"
            >
              <polyline points="22 12 18 12 15 21 9 3 6 12 2 12" />
            </svg>
            <span style={{ color: '#94A3B8', fontSize: 13 }}>Productive time tracked</span>
          </div>
          <span style={{ color: '#22C55E', fontWeight: 600, fontSize: 15 }}>
            {formatHM(totalSecs)}
          </span>
        </div>
      )}

      {loading ? (
        <div className="empty-state">Loading…</div>
      ) : stats.length === 0 ? (
        <div className="card empty-state">No app activity recorded for this day.</div>
      ) : (
        <div style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
          {stats.map((s, i) => {
            const pct = Math.round((s.duration_secs / maxSecs) * 100);
            const share = totalSecs > 0 ? Math.round((s.duration_secs / totalSecs) * 100) : 0;
            const rgb = cardColors[s.app_name];
            const accentColor = rgb ? `rgb(${rgb})` : i === 0 ? '#22C55E' : '#334155';
            const shadowStyle = rgb ? { boxShadow: `0 4px 24px -6px rgba(${rgb}, 0.45)` } : {};

            return (
              <div
                key={s.app_name}
                className="card"
                style={{ padding: '14px 18px', transition: 'box-shadow 0.4s ease', ...shadowStyle }}
              >
                <div
                  style={{
                    display: 'flex',
                    alignItems: 'center',
                    justifyContent: 'space-between',
                    marginBottom: 10,
                  }}
                >
                  <div style={{ display: 'flex', alignItems: 'center', gap: 10 }}>
                    {/* Rank badge */}
                    <span
                      style={{
                        width: 22,
                        height: 22,
                        borderRadius: 6,
                        background: i === 0 ? 'rgba(34,197,94,0.15)' : 'rgba(100,116,139,0.12)',
                        color: i === 0 ? '#22C55E' : '#64748B',
                        fontSize: 11,
                        fontWeight: 700,
                        display: 'flex',
                        alignItems: 'center',
                        justifyContent: 'center',
                        flexShrink: 0,
                      }}
                    >
                      {i + 1}
                    </span>
                    {/* App icon (real icon → favicon fallback → letter avatar) */}
                    <AppIconImg appName={s.app_name} onColorReady={handleColorReady} />
                    {/* Human-readable app name */}
                    <div style={{ display: 'flex', flexDirection: 'column', gap: 1 }}>
                      <span style={{ fontSize: 14, fontWeight: 500, color: '#E2E8F0' }}>
                        {getDisplayName(s.app_name)}
                      </span>
                      {import.meta.env.DEV && s.exe_path && (
                        <span
                          style={{
                            fontSize: 10,
                            color: '#475569',
                            fontFamily: 'monospace',
                            lineHeight: 1.3,
                          }}
                        >
                          {s.exe_path}
                        </span>
                      )}
                    </div>
                  </div>
                  <div style={{ display: 'flex', alignItems: 'center', gap: 10 }}>
                    <span style={{ color: '#64748B', fontSize: 12 }}>{share}%</span>
                    <span
                      style={{
                        color: '#F8FAFC',
                        fontWeight: 600,
                        fontSize: 14,
                        minWidth: 52,
                        textAlign: 'right',
                      }}
                    >
                      {formatHM(s.duration_secs)}
                    </span>
                  </div>
                </div>
                {/* Progress bar */}
                <div
                  style={{ height: 4, background: '#1E293B', borderRadius: 2, overflow: 'hidden' }}
                >
                  <div
                    style={{
                      height: '100%',
                      width: `${pct}%`,
                      background: accentColor,
                      borderRadius: 2,
                      transition: 'width 0.3s ease, background 0.4s ease',
                    }}
                  />
                </div>
              </div>
            );
          })}
        </div>
      )}
    </div>
  );
}
