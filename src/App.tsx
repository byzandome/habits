import { useState, lazy, Suspense } from 'react';
import { NavBar } from './components/NavBar';
import { Dashboard } from './components/Dashboard';
import { useTracker } from './hooks/useTracker';
import './App.css';

// Lazy-load non-default tabs — they're parsed only when first visited
const History  = lazy(() => import('./components/History').then(m => ({ default: m.History })));
const Sessions = lazy(() => import('./components/Sessions').then(m => ({ default: m.Sessions })));
const AppUsage = lazy(() => import('./components/AppUsage').then(m => ({ default: m.AppUsage })));
const Settings = lazy(() => import('./components/Settings').then(m => ({ default: m.Settings })));

type Tab = 'dashboard' | 'history' | 'sessions' | 'apps' | 'settings';

function App() {
  const [tab, setTab] = useState<Tab>('dashboard');
  const tracker = useTracker();

  return (
    <div className="app-shell">
      <NavBar activeTab={tab} onTabChange={setTab} tracker={tracker} />
      <main className="content-area">
        {tab === 'dashboard' && <Dashboard tracker={tracker} />}
        <Suspense fallback={null}>
          {tab === 'history'   && <History />}
          {tab === 'sessions'  && <Sessions />}
          {tab === 'apps'      && <AppUsage />}
          {tab === 'settings'  && <Settings />}
        </Suspense>
      </main>
    </div>
  );
}

export default App;
