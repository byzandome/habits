import { useEffect, useState } from 'react';

import { appUsageUseCases } from '../../infrastructure/container';
import { getFallbackDomain } from '../meta/app.meta';

/** Module-level icon cache — survives re-renders, cleared on explicit cache bust. */
export const iconCache = new Map<string, string | null>();
const iconPending = new Set<string>();

/** Clear the in-memory icon cache (call alongside appUsageUseCases.clearIconCache()). */
export function clearIconMemoryCache(): void {
  iconCache.clear();
  iconPending.clear();
}

/**
 * Returns the display icon URL for `appName`:
 *   1. Native app icon via Tauri (data URI)
 *   2. DuckDuckGo favicon URL (fallback domain from app.meta)
 *   3. null → caller should render a letter avatar
 */
export function useAppIcon(appName: string): string | null {
  const [icon, setIcon] = useState<string | null>(() => iconCache.get(appName) ?? null);

  useEffect(() => {
    if (iconCache.has(appName) || iconPending.has(appName)) return;
    iconPending.add(appName);

    appUsageUseCases
      .getAppIcon(appName)
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
