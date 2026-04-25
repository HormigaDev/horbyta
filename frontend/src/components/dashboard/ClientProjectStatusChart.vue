<template>
  <v-chart v-if="option" :option="option" autoresize />
  <div v-else class="chart-empty">{{ emptyLabel }}</div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import VChart from 'vue-echarts';

import type { ClientProjectStatusRow } from 'src/types/dashboard';
import { ensureEChartsRegistered } from 'src/composables/dashboard/registerECharts';
import {
  baseTooltip,
  pickColor,
  type DashboardEChartsOption,
} from 'src/composables/dashboard/chartHelpers';
import type { ChartPalette } from 'src/composables/dashboard/useChartPalette';

interface Props {
  rows: ClientProjectStatusRow[];
  palette: ChartPalette;
  emptyLabel: string;
  labelForStatus: (status: ClientProjectStatusRow['status']) => string;
}

const props = defineProps<Props>();

ensureEChartsRegistered();

const option = computed<DashboardEChartsOption | null>(() => {
  if (props.rows.length === 0) {
    return null;
  }

  return {
    color: props.palette.palette,
    tooltip: {
      trigger: 'item',
      ...baseTooltip(props.palette),
    },
    legend: {
      bottom: 0,
      textStyle: { color: props.palette.text },
    },
    series: [
      {
        type: 'pie',
        radius: ['45%', '72%'],
        avoidLabelOverlap: true,
        itemStyle: {
          borderColor: props.palette.tooltipBg,
          borderWidth: 2,
        },
        label: {
          color: props.palette.text,
          formatter: '{b}: {c}',
        },
        data: props.rows.map((row, idx) => ({
          value: row.total_projects,
          name: props.labelForStatus(row.status),
          itemStyle: { color: pickColor(props.palette, idx) },
        })),
      },
    ],
  };
});
</script>

<style scoped lang="scss">
.chart-empty {
  height: 100%;
  min-height: 180px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--text-muted);
  font-size: 13px;
}
</style>
