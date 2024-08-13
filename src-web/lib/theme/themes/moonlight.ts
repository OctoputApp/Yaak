import type { YaakTheme } from '../window';
import { YaakColor } from '../yaakColor';

export const colors = {
  lightRed: '#ff98a4',
  red: '#ff757f',
  darkRed: '#ff5370',
  lightOrange: '#f8b576',
  orange: '#ff966c',
  darkOrange: '#fc7b7b',
  yellow: '#ffc777',
  green: '#c3e88d',
  lightTeal: '#7af8ca',
  teal: '#3ad7c7',
  lightCyan: '#b4f9f8',
  cyan: '#78dbff',
  sky: '#60bdff',
  blue: '#7cafff',
  darkBlue: '#3d6fe0',
  darkestBlue: '#3b63cf',
  indigo: '#af9fff',
  purple: '#c4a2ff',
  pink: '#fca7ea',
  darkPink: '#fd8aca',
  saturatedGray: '#7a88cf',
  desaturatedGray: '#979bb6',
  gray11: '#d5def8',
  gray10: '#c8d3f5',
  gray9: '#b4c2f0',
  gray8: '#a9b8e8',
  gray7: '#828bb8',
  gray6: '#444a73',
  gray5: '#2f334d',
  gray4: '#222436',
  gray3: '#1e2030',
  gray2: '#191a2a',
  gray1: '#131421',
} as const;

const moonlightDefault: YaakTheme = {
  id: 'moonlight',
  name: 'Moonlight',
  surface: new YaakColor('#222436', 'dark'),
  text: new YaakColor('#d5def8', 'dark'),
  textSubtle: new YaakColor('#828bb8', 'dark'),
  textSubtlest: new YaakColor('hsl(232,26%,43%)', 'dark'),
  primary: new YaakColor(colors.purple, 'dark'),
  secondary: new YaakColor(colors.desaturatedGray, 'dark'),
  info: new YaakColor(colors.blue, 'dark'),
  success: new YaakColor(colors.teal, 'dark'),
  notice: new YaakColor(colors.yellow, 'dark'),
  warning: new YaakColor(colors.orange, 'dark'),
  danger: new YaakColor(colors.red, 'dark'),
  components: {
    appHeader: {
      surface: new YaakColor(colors.gray3, 'dark'),
    },
    sidebar: {
      surface: new YaakColor(colors.gray3, 'dark'),
    },
  },
};

export const moonlight = [moonlightDefault];
