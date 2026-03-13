import { useState } from 'react';
import { NavBar } from './components/NavBar';
import { Dashboard } from './components/Dashboard';
import { History } from './components/History';
import { Sessions } from './components/Sessions';
import { Settings } from './components/Settings';
import { useTracker } from './hooks/useTracker';
import './App.css';

type Tab = 'dashboard' | 'history' | 'sessions' | 'settings';

function App() {
  const [tab, setTab] = useState<Tab>('dashboard');
  const tracker = useTracker();

  return (
    <div className="app-shell">
      <NavBar activeTab={tab} onTabChange={setTab} tracker={tracker} />
      <main className="content-area">
        {tab === 'dashboard' && <Dashboard tracker={tracker} />}
        {tab === 'history'   && <History />}
        {tab === 'sessions'  && <Sessions />}
        {tab === 'settings'  && <Settings />}
      </main>
    </div>
  );
}

export default App;
