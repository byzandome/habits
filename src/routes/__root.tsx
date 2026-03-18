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
    <main className="h-screen w-screen overflow-hidden">
      <div className="flex h-full w-full">
        <NavBar />
        <div className="flex-1 overflow-y-auto">
          <Outlet />
        </div>
      </div>
      {import.meta.env.DEV && <TanStackRouterDevtools position='bottom-right' />}
    </main>
  );
}
