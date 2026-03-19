import { t, type Dictionary } from 'intlayer';

const navbarContent = {
  key: 'navbar',
  content: {
    dashboard: t({
      en: 'Dashboard',
      fr: 'Tableau de bord',
      es: 'Tablero',
    }),
    timeline: t({
      en: 'Timeline',
      fr: 'Chronologie',
      es: 'Cronología',
    }),
    appUsage: t({
      en: 'App Usage',
      fr: "Utilisation de l'application",
      es: 'Uso de la aplicación',
    }),
    settings: t({
      en: 'Settings',
      fr: 'Paramètres',
      es: 'Configuración',
    }),
  },
} satisfies Dictionary;

export default navbarContent;
