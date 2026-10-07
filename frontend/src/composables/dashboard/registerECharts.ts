import { use } from 'echarts/core';
import { CanvasRenderer } from 'echarts/renderers';
import { BarChart, LineChart, PieChart } from 'echarts/charts';
import {
  GridComponent,
  LegendComponent,
  TitleComponent,
  TooltipComponent,
  AriaComponent,
  LegendScrollComponent,
} from 'echarts/components';

let registered = false;

export function ensureEChartsRegistered(): void {
  if (registered) {
    return;
  }
  use([
    CanvasRenderer,
    BarChart,
    LineChart,
    PieChart,
    GridComponent,
    LegendComponent,
    TitleComponent,
    TooltipComponent,
    AriaComponent,
    LegendScrollComponent,
  ]);
  registered = true;
}
