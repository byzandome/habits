import { useEffect } from 'react';

import { createRootRoute, Outlet } from '@tanstack/react-router';
import { TanStackRouterDevtools } from '@tanstack/router-devtools';

import { NavBar } from '../components/NavBar';
import { useTrackerStore } from '../store/tracker';

export const Route = createRootRoute({ component: RootComponent });

function RootComponent() {
  const init = useTrackerStore((s) => s.init);

  useEffect(() => {
    let cleanup: (() => void) | undefined;
    init().then((fn) => { cleanup = fn; });
    return () => { cleanup?.(); };
  }, [init]);

  return (
    <>
      <div className="app-shell">
        <NavBar />
        <main className="content-area">
          <Outlet />
        </main>
      </div>
      {import.meta.env.DEV && <TanStackRouterDevtools />}
    </>
  );
}
