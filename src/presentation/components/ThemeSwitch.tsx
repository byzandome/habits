import { useThemeStore } from '@/presentation/store/theme';
import { Moon, Sun } from 'lucide-react';
import { useCallback } from 'react';
import { Button } from './ui/button';

export default function ThemeSwitch() {
  const { resolvedTheme, theme, setTheme } = useThemeStore();

  const toggleTheme = useCallback(() => {
    setTheme(theme === 'light' ? 'dark' : 'light');
  }, [theme, setTheme]);

  return (
    <Button variant="outline" onClick={toggleTheme} className="theme-switch">
      {resolvedTheme === 'light' ? <Sun /> : <Moon />}
    </Button>
  );
}
