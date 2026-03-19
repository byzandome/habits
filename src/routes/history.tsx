import { createFileRoute } from '@tanstack/react-router';

import { History } from '@/presentation/pages/History';

export const Route = createFileRoute('/history')({ component: History });
