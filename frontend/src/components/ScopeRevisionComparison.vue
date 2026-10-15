<template>
  <div class="scope-comparison">
    <q-select
      v-model="otherId"
      :options="
        revisions
          .filter((r) => r.revision !== current.revision)
          .map((r) => ({ label: t('scope.revision', { version: r.revision }), value: r.revision }))
      "
      emit-value
      map-options
      :label="t('scope.comparisonRevision')"
    />
    <template v-if="other">
      <p>
        {{ t('scope.comparisonHint', { from: other.revision, to: current.revision }) }}
      </p>
      <q-expansion-item v-for="change in changes" :key="change.title" :label="change.title">
        <pre
          class="scope-diff"
        ><span v-for="(line,index) in change.lines" :key="index" :class="line.kind">{{ line.text }}{{ '\n' }}</span></pre>
      </q-expansion-item>
      <p v-if="!changes.length">{{ t('scope.noDifferences') }}</p>
    </template>
  </div>
</template>
<script setup lang="ts">
import { useI18n } from 'vue-i18n';
const { t } = useI18n();
import { computed, ref, watch } from 'vue';
import type { ScopeRevision } from 'src/types/scope';
const props = defineProps<{ current: ScopeRevision; revisions: ScopeRevision[] }>();
const otherId = ref<number | null>(null);
watch(
  () => props.current.revision,
  () => {
    otherId.value =
      props.revisions.find((r) => r.revision !== props.current.revision)?.revision ?? null;
  },
  { immediate: true },
);
const other = computed(() => props.revisions.find((r) => r.revision === otherId.value));
function lines(before: string, after: string) {
  const a = before.split('\n'),
    b = after.split('\n');
  let start = 0,
    end = 0;
  while (start < a.length && start < b.length && a[start] === b[start]) start++;
  while (
    end < a.length - start &&
    end < b.length - start &&
    a[a.length - 1 - end] === b[b.length - 1 - end]
  )
    end++;
  return [
    ...a
      .slice(Math.max(0, start - 2), start)
      .map((text) => ({ kind: 'context', text: `  ${text}` })),
    ...a.slice(start, a.length - end).map((text) => ({ kind: 'removed', text: `− ${text}` })),
    ...b.slice(start, b.length - end).map((text) => ({ kind: 'added', text: `+ ${text}` })),
    ...b
      .slice(b.length - end, b.length - end + 2)
      .map((text) => ({ kind: 'context', text: `  ${text}` })),
  ];
}
const changes = computed(() => {
  if (!other.value) return [];
  const result: { title: string; lines: ReturnType<typeof lines> }[] = [];
  const push = (title: string, a: unknown, b: unknown) => {
    const before = typeof a === 'string' ? a : JSON.stringify(a, null, 2),
      after = typeof b === 'string' ? b : JSON.stringify(b, null, 2);
    if (before !== after) result.push({ title, lines: lines(before ?? '', after ?? '') });
  };
  const a = other.value,
    b = props.current;
  push(t('scope.includedChanges'), a.approved_changes, b.approved_changes);
  push(
    t('scope.amendmentReason'),
    a.document.amendment_reason ?? '',
    b.document.amendment_reason ?? '',
  );
  for (const key of [
    'deliverables',
    'criteria',
    'restrictions',
    'exclusions',
    'requirements',
    'precedence',
  ] as const)
    push(
      {
        deliverables: t('scope.deliverables'),
        criteria: t('scope.criteria'),
        restrictions: t('scope.restrictions'),
        exclusions: t('scope.exclusions'),
        requirements: t('scope.requirements'),
        precedence: t('scope.precedenceTitle'),
      }[key],
      a.document[key] ?? '',
      b.document[key] ?? '',
    );
  const docs = new Map(
    [...(a.document.documents ?? []), ...(b.document.documents ?? [])].map((d) => [d.id, d.title]),
  );
  for (const [id, title] of docs) {
    const before = a.document.documents?.find((d) => d.id === id),
      after = b.document.documents?.find((d) => d.id === id);
    push(t('scope.contentDiff', { title }), before?.content ?? '', after?.content ?? '');
    const meta = (d: typeof before) =>
      d
        ? {
            title: d.title,
            role: d.role,
            section: d.section,
            requirement_id: d.requirement_id,
            source_name: d.source_name,
            source_hash: d.source_hash,
          }
        : null;
    push(t('scope.metadataDiff', { title }), meta(before), meta(after));
  }
  push(t('scope.includedAttachments'), a.attachment_snapshot, b.attachment_snapshot);
  push(t('scope.financialTerms'), a.billing_snapshot, b.billing_snapshot);
  push(t('scope.datesMilestones'), a.milestone_snapshot, b.milestone_snapshot);
  push(t('scope.tasks'), a.task_snapshot, b.task_snapshot);
  return result;
});
</script>
<style scoped>
.scope-diff {
  white-space: pre-wrap;
  overflow-wrap: anywhere;
  padding: 12px;
  background: var(--surface-muted);
}
.removed {
  color: var(--q-negative);
}
.added {
  color: var(--q-positive);
}
</style>
