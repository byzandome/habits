// Composition root — wires repository implementations to use-case factories.
// The rest of the app imports pre-built use-case objects from here.

import { createAppUsageUseCases } from '../application/use-cases/app-usage';
import { createHistoryUseCases } from '../application/use-cases/history';
import { createSessionUseCases } from '../application/use-cases/sessions';
import { createSettingsUseCases } from '../application/use-cases/settings';
import { createTrackingUseCases } from '../application/use-cases/tracking';
import { TauriAppUsageRepository } from './tauri/app-usage.repository';
import { TauriHistoryRepository } from './tauri/history.repository';
import { TauriSessionRepository } from './tauri/session.repository';
import { TauriSettingsRepository } from './tauri/settings.repository';
import { TauriTrackingRepository } from './tauri/tracking.repository';

const trackingRepository = new TauriTrackingRepository();
const sessionRepository = new TauriSessionRepository();
const historyRepository = new TauriHistoryRepository();
const appUsageRepository = new TauriAppUsageRepository();
const settingsRepository = new TauriSettingsRepository();

export const trackingUseCases = createTrackingUseCases(trackingRepository);
export const sessionUseCases = createSessionUseCases(sessionRepository);
export const historyUseCases = createHistoryUseCases(historyRepository);
export const appUsageUseCases = createAppUsageUseCases(appUsageRepository);
export const settingsUseCases = createSettingsUseCases(settingsRepository);
