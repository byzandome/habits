import { useCallback, type ReactElement } from 'react';

import { Link } from '@tanstack/react-router';
import { CirclePile, Cog, History, House } from 'lucide-react';
import ThemeSwitch from './ThemeSwitch';
import { useIntlayer } from 'react-intlayer';
import navbarContent from '@/content/navbar.content';

type NavItem = {
  to: string;
  label: keyof typeof navbarContent['content'];
  icon: ReactElement;
};

const ICON_SIZE = 18;

const NAV_ITEMS: NavItem[] = [
  {
    to: '/',
    label: 'dashboard',
    icon: <House size={ICON_SIZE} />
  },
  {
    to: '/history',
    label: 'timeline',
    icon: <History size={ICON_SIZE} />,
  },

  {
    to: '/apps',
    label: 'appUsage',
    icon: <CirclePile size={ICON_SIZE} />,
  },
  {
    to: '/settings',
    label: 'settings',
    icon: <Cog size={ICON_SIZE} />,
  },
];


export function NavBar() {
  const content = useIntlayer("navbar");

  const getActiveOptions = useCallback((to: string) => {
    return to === '/' ? { exact: true } : undefined;
  }, []);

  return (
    <aside className="w-55 bg-background flex flex-col px-4 py-6 border-r gap-2">
      {/* Logo */}
      <div className="flex items-center justify-between pt-2 px-2.5 pb-4 w-full">
        <div className='flex items-center gap-2.5'>
          <div className="w-8 h-8 flex items-center justify-center rounded-full bg-green-100">
            <img src="/images/habits.png" alt="Habits" />
          </div>
          <span className="text-foreground font-bold">Habits</span>
        </div>
        <ThemeSwitch />
      </div>

      {/* Navigation */}
      <nav className="flex flex-col gap-1 ">
        {NAV_ITEMS.map(({ to, label, icon }) => (
          <Link
            key={to}
            to={to}
            className="flex items-center text-foreground gap-2.5 px-3 py-2 rounded-md border-0 bg-transparent hover:bg-gray-200/10 transition-colors text-sm font-medium cursor-pointer text-left [&.active]:bg-primary [&.active]:text-primary-foreground"
            activeOptions={getActiveOptions(to)}
          >
            <span className="flex items-center">{icon}</span>
            <span>{content[label]}</span>
          </Link>
        ))}
      </nav>
    </aside>
  );
}
