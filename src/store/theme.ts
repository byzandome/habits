import { create } from 'zustand';
import { persist } from 'zustand/middleware';

export type ThemeMode = 'light' | 'dark' | 'system';
export type ResolvedTheme = 'light' | 'dark';

interface ThemeStore {
  /** User-selected preference. */
  theme: ThemeMode;
  /** Actual theme in use after resolving 'system'. */
  resolvedTheme: ResolvedTheme;
  setTheme: (theme: ThemeMode) => void;
}

function getSystemTheme(): ResolvedTheme {
  return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
}

function resolveTheme(theme: ThemeMode): ResolvedTheme {
  return theme === 'system' ? getSystemTheme() : theme;
}

function applyTheme(resolved: ResolvedTheme) {
  document.documentElement.classList.toggle('dark', resolved === 'dark');
}

export const useThemeStore = create<ThemeStore>()(
  persist(
    (set) => ({
      theme: 'system' as ThemeMode,
      resolvedTheme: resolveTheme('system'),

      setTheme: (theme) => {
        const resolved = resolveTheme(theme);
        applyTheme(resolved);
        set({ theme, resolvedTheme: resolved });
      },
    }),
    {
      name: 'habits-theme',
      // Re-apply the theme class and recalculate resolvedTheme after hydration.
      onRehydrateStorage: () => (state) => {
        if (!state) return;
        const resolved = resolveTheme(state.theme);
        state.resolvedTheme = resolved;
        applyTheme(resolved);
      },
    },
  ),
);

// Keep 'system' mode in sync with OS preference changes at runtime.
window
  .matchMedia('(prefers-color-scheme: dark)')
  .addEventListener('change', () => {
    const { theme, setTheme } = useThemeStore.getState();
    if (theme === 'system') setTheme('system');
  });
