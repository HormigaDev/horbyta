import type { BarSeriesOption, LineSeriesOption, PieSeriesOption } from 'echarts/charts';
import type {
  GridComponentOption,
  LegendComponentOption,
  TitleComponentOption,
  TooltipComponentOption,
  AriaComponentOption,
} from 'echarts/components';
import type { ComposeOption } from 'echarts/core';

import type { ChartPalette } from './useChartPalette';

export type DashboardEChartsOption = ComposeOption<
  | BarSeriesOption
  | LineSeriesOption
  | PieSeriesOption
  | GridComponentOption
  | LegendComponentOption
  | TitleComponentOption
  | TooltipComponentOption
  | AriaComponentOption
>;

export function baseTooltip(palette: ChartPalette): TooltipComponentOption {
  return {
    backgroundColor: palette.tooltipBg,
    borderColor: palette.grid,
    textStyle: { color: palette.text },
    confine: true,
    renderMode: 'richText',
  };
}

/** Color identity survives a change in row order or applied filters. */
export function seriesIndex(identity: string): number {
  let hash = 2166136261;
  for (const char of identity) hash = Math.imul(hash ^ char.charCodeAt(0), 16777619) >>> 0;
  return hash;
}

export function chartAccessibility(palette: ChartPalette, description: string) {
  return {
    animation: !palette.reducedMotion,
    aria: { enabled: true, label: { description }, decal: { show: true } },
  };
}

export function chronologicalDates(dates: string[]): string[] {
  return Array.from(new Set(dates)).sort((a, b) => a.localeCompare(b));
}

export function pickColor(palette: ChartPalette, index: number): string {
  return palette.palette[index % palette.palette.length] ?? palette.primary;
}
