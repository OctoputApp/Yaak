import parseColor from 'parse-color';

export class Color {
  private theme: 'dark' | 'light' = 'light';

  private hue: number = 0;
  private saturation: number = 0;
  private lightness: number = 0;
  private alpha: number = 1;

  constructor(cssColor: string, theme: 'dark' | 'light') {
    try {
      const { hsla } = parseColor(cssColor || '');
      this.hue = hsla[0];
      this.saturation = hsla[1];
      this.lightness = hsla[2];
      this.alpha = hsla[3] ?? 1;
      this.theme = theme;
    } catch (err) {
      console.log('Failed to parse CSS color', cssColor, err);
    }
  }

  static transparent(): Color {
    return new Color('rgba(0, 0, 0, 0.1)', 'light');
  }

  private clone(): Color {
    return new Color(this.css(), this.theme);
  }

  lower(mod: number): Color {
    return this.theme === 'dark' ? this._darken(mod) : this._lighten(mod);
  }

  lowerTo(value: number): Color {
    return this.theme === 'dark'
      ? this._darken(1)._lighten(value)
      : this._lighten(1)._darken(1 - value);
  }

  lift(mod: number): Color {
    return this.theme === 'dark' ? this._lighten(mod) : this._darken(mod);
  }

  liftTo(value: number): Color {
    return this.theme === 'dark'
      ? this._lighten(1)._darken(1 - value)
      : this._darken(1)._lighten(value);
  }

  translucify(mod: number): Color {
    const c = this.clone();
    c.alpha = c.alpha - c.alpha * mod;
    return c;
  }

  desaturate(mod: number): Color {
    const c = this.clone();
    c.saturation = c.saturation - c.saturation * mod;
    return c;
  }

  saturate(mod: number): Color {
    const c = this.clone();
    c.saturation = this.saturation + (100 - this.saturation) * mod;
    return c;
  }

  lighterThan(c: Color): boolean {
    return this.lightness > c.lightness;
  }

  css(): string {
    // If opacity is 1, allow for Tailwind modification
    const h = Math.round(this.hue);
    const s = Math.round(this.saturation);
    const l = Math.round(this.lightness);
    const a = Math.round(this.alpha * 100) / 100;
    return `hsla(${h}, ${s}%, ${l}%, ${a})`;
  }

  private _lighten(mod: number): Color {
    const c = this.clone();
    c.lightness = this.lightness + (100 - this.lightness) * mod;
    return c;
  }

  private _darken(mod: number): Color {
    const c = this.clone();
    c.lightness = this.lightness - this.lightness * mod;
    return c;
  }
}
