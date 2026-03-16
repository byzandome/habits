import { createRootRoute, Outlet } from '@tanstack/react-router';
import { TanStackRouterDevtools } from '@tanstack/router-devtools';

import { NavBar } from '../components/NavBar';
import { TrackerProvider } from '../context/tracker';
import { useTracker } from '../hooks/useTracker';

export const Route = createRootRoute({ component: RootComponent });

function RootComponent() {
  const tracker = useTracker();

  return (
    <TrackerProvider value={tracker}>
      <div className="app-shell">
        <NavBar />
        <main className="content-area">
          <Outlet />
        </main>
      </div>
      {import.meta.env.DEV && <TanStackRouterDevtools />}
    </TrackerProvider>
  );
}
