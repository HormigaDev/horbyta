<template>
  <div v-if="option" class="dashboard-chart">
    <div class="chart-visualization"><v-chart :option="option" autoresize /></div>
    <ChartDataTable
      :caption="t('clientDashboard.projectStatus.title')"
      :headers="[t('common.status'), t('layout.drawer.projects')]"
      :rows="rows.map((row) => [labelForStatus(row.status), row.total_projects])"
    />
  </div>
  <div v-else class="chart-empty">{{ emptyLabel }}</div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import VChart from 'vue-echarts';
import { useI18n } from 'vue-i18n';
import ChartDataTable from './ChartDataTable.vue';

import type { ClientProjectStatusRow } from 'src/types/dashboard';
import { ensureEChartsRegistered } from 'src/composables/dashboard/registerECharts';
import {
  baseTooltip,
  pickColor,
  seriesIndex,
  chartAccessibility,
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
const { t } = useI18n();

ensureEChartsRegistered();

const option = computed<DashboardEChartsOption | null>(() => {
  if (props.rows.length === 0) {
    return null;
  }

  return {
    ...chartAccessibility(
      props.palette,
      t('ui.chartDescription', { title: t('clientDashboard.projectStatus.title') }),
    ),
    color: props.palette.palette,
    tooltip: {
      trigger: 'item',
      ...baseTooltip(props.palette),
    },
    legend: {
      type: 'scroll',
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
          show: false,
        },
        data: props.rows.map((row) => ({
          value: row.total_projects,
          name: props.labelForStatus(row.status),
          itemStyle: { color: pickColor(props.palette, seriesIndex(row.status)) },
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
