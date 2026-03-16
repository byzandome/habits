import type { TrackerState } from '../hooks/useTracker';

import { createContext, useContext } from 'react';

const DEFAULT: TrackerState = {
  status: 'productive',
  sessionDurationSecs: 0,
  productiveSecs: 0,
  idleSecs: 0,
  lockedSecs: 0,
};

const TrackerContext = createContext<TrackerState>(DEFAULT);

export const TrackerProvider = TrackerContext.Provider;

export function useTrackerContext(): TrackerState {
  return useContext(TrackerContext);
}
