const LIGHT_CONTRAST_TEXT = '#FFFFFF';
const DARK_CONTRAST_TEXT = '#000000';
const HEX_COLOR_REGEX = /^#(?:[0-9A-F]{3}|[0-9A-F]{6})$/i;

const expandShortHex = (hexColor: string): string => {
  if (hexColor.length === 4) {
    const r = hexColor[1];
    const g = hexColor[2];
    const b = hexColor[3];
    return `#${r}${r}${g}${g}${b}${b}`;
  }

  return hexColor;
};

const normalizeHexColor = (hexColor: string): string | null => {
  if (!hexColor) return null;
  const candidate = hexColor.trim();
  if (!HEX_COLOR_REGEX.test(candidate)) {
    return null;
  }

  return expandShortHex(candidate.toUpperCase());
};

export const getTagTextContrastColor = (hexColor: string): string => {
  const normalizedHexColor = normalizeHexColor(hexColor);
  if (!normalizedHexColor) {
    return DARK_CONTRAST_TEXT;
  }

  const red = Number.parseInt(normalizedHexColor.slice(1, 3), 16);
  const green = Number.parseInt(normalizedHexColor.slice(3, 5), 16);
  const blue = Number.parseInt(normalizedHexColor.slice(5, 7), 16);
  const linear = (channel: number) => {
    const value = channel / 255;
    return value <= 0.04045 ? value / 12.92 : ((value + 0.055) / 1.055) ** 2.4;
  };
  const luminance = 0.2126 * linear(red) + 0.7152 * linear(green) + 0.0722 * linear(blue);
  return (luminance + 0.05) / 0.05 >= 1.05 / (luminance + 0.05)
    ? DARK_CONTRAST_TEXT
    : LIGHT_CONTRAST_TEXT;
};
