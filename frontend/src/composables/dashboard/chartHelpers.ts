import type { BarSeriesOption, LineSeriesOption, PieSeriesOption } from 'echarts/charts';
import type {
  GridComponentOption,
  LegendComponentOption,
  TitleComponentOption,
  TooltipComponentOption,
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
>;

export function baseTooltip(palette: ChartPalette): TooltipComponentOption {
  return {
    backgroundColor: palette.tooltipBg,
    borderColor: palette.grid,
    textStyle: { color: palette.text },
  };
}

export function pickColor(palette: ChartPalette, index: number): string {
  return palette.palette[index % palette.palette.length] ?? palette.primary;
}
