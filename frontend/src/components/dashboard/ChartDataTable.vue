<template>
  <details class="chart-data">
    <summary>
      {{ t('ui.chartData') }} <span class="text-secondary">— {{ rows.length }}</span>
    </summary>
    <div class="chart-data__scroll" role="region" :aria-label="caption" tabindex="0">
      <table>
        <caption>
          {{
            caption
          }}
        </caption>
        <thead>
          <tr>
            <th v-for="(header, index) in headers" :key="index" scope="col">{{ header }}</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="(row, index) in rows" :key="index">
            <td v-for="(value, cell) in row" :key="cell">{{ value }}</td>
          </tr>
        </tbody>
      </table>
    </div>
  </details>
</template>
<script setup lang="ts">
import { useI18n } from 'vue-i18n';
defineProps<{ caption: string; headers: string[]; rows: (string | number)[][] }>();
const { t } = useI18n();
</script>
