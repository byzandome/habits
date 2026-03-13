// Maps known process file-stems to human-readable names and a favicon domain
// used as fallback when the Windows icon cannot be extracted.

interface AppMeta {
  displayName: string;
  /** Domain used by `https://icons.duckduckgo.com/ip3/{domain}.ico` fallback. */
  fallbackDomain?: string;
}

const APP_META: Record<string, AppMeta> = {
  // ── Browsers ──────────────────────────────────────────────────────────────
  msedge:  { displayName: 'Microsoft Edge',    fallbackDomain: 'microsoft.com'        },
  chrome:  { displayName: 'Google Chrome',     fallbackDomain: 'google.com'           },
  firefox: { displayName: 'Firefox',           fallbackDomain: 'firefox.com'          },
  brave:   { displayName: 'Brave',             fallbackDomain: 'brave.com'            },
  opera:   { displayName: 'Opera',             fallbackDomain: 'opera.com'            },
  vivaldi: { displayName: 'Vivaldi',           fallbackDomain: 'vivaldi.com'          },
  arc:     { displayName: 'Arc',               fallbackDomain: 'arc.net'             },

  // ── Microsoft Office / 365 ────────────────────────────────────────────────
  winword:  { displayName: 'Microsoft Word',       fallbackDomain: 'microsoft.com' },
  excel:    { displayName: 'Microsoft Excel',      fallbackDomain: 'microsoft.com' },
  powerpnt: { displayName: 'Microsoft PowerPoint', fallbackDomain: 'microsoft.com' },
  outlook:  { displayName: 'Microsoft Outlook',    fallbackDomain: 'microsoft.com' },
  onenote:  { displayName: 'Microsoft OneNote',    fallbackDomain: 'microsoft.com' },
  msaccess: { displayName: 'Microsoft Access',     fallbackDomain: 'microsoft.com' },
  mspub:    { displayName: 'Microsoft Publisher',  fallbackDomain: 'microsoft.com' },

  // ── Collaboration / Communication ─────────────────────────────────────────
  msteams:       { displayName: 'Microsoft Teams', fallbackDomain: 'microsoft.com' },
  teams:         { displayName: 'Microsoft Teams', fallbackDomain: 'microsoft.com' },
  'ms-teamsupd': { displayName: 'Microsoft Teams', fallbackDomain: 'microsoft.com' },
  slack:         { displayName: 'Slack',            fallbackDomain: 'slack.com'    },
  discord:       { displayName: 'Discord',          fallbackDomain: 'discord.com'  },
  zoom:          { displayName: 'Zoom',             fallbackDomain: 'zoom.us'      },
  skype:         { displayName: 'Skype',            fallbackDomain: 'skype.com'    },
  telegram:      { displayName: 'Telegram',         fallbackDomain: 'telegram.org' },
  whatsapp:      { displayName: 'WhatsApp',         fallbackDomain: 'whatsapp.com' },
  signal:        { displayName: 'Signal',           fallbackDomain: 'signal.org'   },

  // ── Development ───────────────────────────────────────────────────────────
  code:                { displayName: 'Visual Studio Code', fallbackDomain: 'code.visualstudio.com' },
  devenv:              { displayName: 'Visual Studio',      fallbackDomain: 'visualstudio.microsoft.com' },
  windowsterminal:     { displayName: 'Windows Terminal',   fallbackDomain: 'microsoft.com' },
  'windows terminal':  { displayName: 'Windows Terminal',   fallbackDomain: 'microsoft.com' },
  powershell:          { displayName: 'PowerShell',         fallbackDomain: 'microsoft.com' },
  cmd:                 { displayName: 'Command Prompt',     fallbackDomain: 'microsoft.com' },
  wsl:                 { displayName: 'WSL',                fallbackDomain: 'microsoft.com' },
  gitkraken:           { displayName: 'GitKraken',          fallbackDomain: 'gitkraken.com' },
  sourcetree:          { displayName: 'Sourcetree',         fallbackDomain: 'sourcetreeapp.com' },
  postman:             { displayName: 'Postman',            fallbackDomain: 'postman.com' },
  insomnia:            { displayName: 'Insomnia',           fallbackDomain: 'insomnia.rest' },
  datagrip:            { displayName: 'DataGrip',           fallbackDomain: 'jetbrains.com' },
  idea:                { displayName: 'IntelliJ IDEA',      fallbackDomain: 'jetbrains.com' },
  webstorm:            { displayName: 'WebStorm',           fallbackDomain: 'jetbrains.com' },
  pycharm:             { displayName: 'PyCharm',            fallbackDomain: 'jetbrains.com' },
  rider:               { displayName: 'Rider',              fallbackDomain: 'jetbrains.com' },
  clion:               { displayName: 'CLion',              fallbackDomain: 'jetbrains.com' },
  'android studio':    { displayName: 'Android Studio',     fallbackDomain: 'developer.android.com' },
  studio64:            { displayName: 'Android Studio',     fallbackDomain: 'developer.android.com' },
  cursor:              { displayName: 'Cursor',             fallbackDomain: 'cursor.sh' },

  // ── Design ────────────────────────────────────────────────────────────────
  figma:   { displayName: 'Figma',            fallbackDomain: 'figma.com'     },
  xd:      { displayName: 'Adobe XD',         fallbackDomain: 'adobe.com'     },
  ps:      { displayName: 'Adobe Photoshop',  fallbackDomain: 'adobe.com'     },
  ai:      { displayName: 'Adobe Illustrator',fallbackDomain: 'adobe.com'     },
  ae:      { displayName: 'Adobe After Effects', fallbackDomain: 'adobe.com'  },
  pr:      { displayName: 'Adobe Premiere',   fallbackDomain: 'adobe.com'     },

  // ── Productivity ──────────────────────────────────────────────────────────
  notion:   { displayName: 'Notion',       fallbackDomain: 'notion.so'     },
  obsidian: { displayName: 'Obsidian',     fallbackDomain: 'obsidian.md'   },
  'notion calendar': { displayName: 'Notion Calendar', fallbackDomain: 'notion.so' },

  // ── Media ─────────────────────────────────────────────────────────────────
  spotify: { displayName: 'Spotify',     fallbackDomain: 'spotify.com'   },
  vlc:     { displayName: 'VLC',          fallbackDomain: 'videolan.org'  },

  // ── Windows built-ins ─────────────────────────────────────────────────────
  explorer:  { displayName: 'File Explorer',  fallbackDomain: 'microsoft.com' },
  taskmgr:   { displayName: 'Task Manager',   fallbackDomain: 'microsoft.com' },
  notepad:   { displayName: 'Notepad',        fallbackDomain: 'microsoft.com' },
  mspaint:   { displayName: 'Paint',          fallbackDomain: 'microsoft.com' },
  calc:      { displayName: 'Calculator',     fallbackDomain: 'microsoft.com' },
  snippingtool: { displayName: 'Snipping Tool', fallbackDomain: 'microsoft.com' },
  'notepad++': { displayName: 'Notepad++',    fallbackDomain: 'notepad-plus-plus.org' },
};

/** Returns the human-readable display name for a process stem. */
export function getDisplayName(appName: string): string {
  const meta = APP_META[appName.toLowerCase()];
  if (meta) return meta.displayName;
  // Capitalise first character as a reasonable fallback
  return appName.charAt(0).toUpperCase() + appName.slice(1);
}

/** Returns the favicon domain for a process stem, if known. */
export function getFallbackDomain(appName: string): string | undefined {
  return APP_META[appName.toLowerCase()]?.fallbackDomain;
}
