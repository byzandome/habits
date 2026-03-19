import { createFileRoute } from '@tanstack/react-router';

import { Settings } from '@/presentation/pages/Settings';

export const Route = createFileRoute('/settings')({ component: Settings });
