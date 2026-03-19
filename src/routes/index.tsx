import { createFileRoute } from '@tanstack/react-router';

import { Dashboard } from '@/presentation/pages/Dashboard';

export const Route = createFileRoute('/')({ component: Dashboard });
