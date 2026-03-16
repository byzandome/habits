import { createFileRoute } from '@tanstack/react-router';

import { AppUsage } from '../pages/AppUsage';

export const Route = createFileRoute('/apps')({ component: AppUsage });
