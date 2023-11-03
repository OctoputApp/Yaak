import type { AppTheme, AppThemeColors } from './theme';
import { generateCSS, toTailwindVariable } from './theme';

export type Appearance = 'dark' | 'light';

enum Theme {
  yaak = 'yaak',
  catppuccin = 'catppuccin',
}

const themes: Record<Theme, AppThemeColors> = {
  yaak: {
    gray: '#6b5b98',
    red: '#ff417b',
    orange: '#fd9014',
    yellow: '#e8d13f',
    green: '#3fd265',
    blue: '#219dff',
    pink: '#ff6dff',
    violet: '#b176ff',
  },
  catppuccin: {
    gray: 'hsl(240, 23%, 47%)',
    red: 'hsl(343, 91%, 74%)',
    orange: 'hsl(23, 92%, 74%)',
    yellow: 'hsl(41, 86%, 72%)',
    green: 'hsl(115, 54%, 65%)',
    blue: 'hsl(217, 92%, 65%)',
    pink: 'hsl(316, 72%, 75%)',
    violet: 'hsl(267, 84%, 70%)',
  },
};

const darkTheme: AppTheme = {
  name: 'Default Dark',
  appearance: 'dark',
  layers: {
    root: {
      blackPoint: 0.2,
      colors: themes.catppuccin,
    },
  },
};

const lightTheme: AppTheme = {
  name: 'Default Light',
  appearance: 'light',
  layers: {
    root: {
      colors: {
        gray: '#7f8fb0',
        red: '#ec3f87',
        orange: '#ff8000',
        yellow: '#e7cf24',
        green: '#00d365',
        blue: '#0090ff',
        pink: '#ea6cea',
        violet: '#ac6cff',
      },
    },
  },
};

export function getAppearance(): Appearance {
  const docAppearance = document.documentElement.getAttribute('data-appearance');
  if (docAppearance === 'dark' || docAppearance === 'light') {
    return docAppearance;
  }
  return getPreferredAppearance();
}

export function setAppearance(a?: Appearance) {
  const appearance = a ?? getPreferredAppearance();
  const theme = appearance === 'dark' ? darkTheme : lightTheme;

  document.documentElement.setAttribute('data-appearance', appearance);
  document.documentElement.setAttribute('data-theme', theme.name);

  let existingStyleEl = document.head.querySelector(`style[data-theme-definition]`);
  if (!existingStyleEl) {
    const styleEl = document.createElement('style');
    document.head.appendChild(styleEl);
    existingStyleEl = styleEl;
  }

  existingStyleEl.textContent = [
    `/* ${darkTheme.name} */`,
    `[data-appearance="dark"] {`,
    ...generateCSS(darkTheme).map(toTailwindVariable),
    '}',
    `/* ${lightTheme.name} */`,
    `[data-appearance="light"] {`,
    ...generateCSS(lightTheme).map(toTailwindVariable),
    '}',
  ].join('\n');
  existingStyleEl.setAttribute('data-theme-definition', '');
}

export function getPreferredAppearance(): Appearance {
  return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
}

export function subscribeToPreferredAppearanceChange(
  cb: (appearance: Appearance) => void,
): () => void {
  const listener = (e: MediaQueryListEvent) => cb(e.matches ? 'dark' : 'light');
  const m = window.matchMedia('(prefers-color-scheme: dark)');
  m.addEventListener('change', listener);
  return () => m.removeEventListener('change', listener);
}
