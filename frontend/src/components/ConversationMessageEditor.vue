<template>
  <q-field
    ref="field"
    :model-value="modelValue"
    outlined
    :disable="disabled"
    :rules="rules"
    bottom-slots
    class="conversation-message-field"
  >
    <template #control="{ id }">
      <div
        :id="id"
        ref="editor"
        class="conversation-message-editor"
        :contenteditable="!disabled"
        role="textbox"
        aria-multiline="true"
        :aria-label="label"
        :data-placeholder="label"
        :aria-expanded="expanded"
        :aria-controls="controls"
        spellcheck="true"
        @input="sync"
        @keydown="keydown"
        @keyup="$emit('keyup', $event)"
        @click="$emit('cursorChange')"
        @paste="paste"
        @drop.prevent
        @beforeinput="beforeInput"
      />
    </template>
  </q-field>
</template>
<script setup lang="ts">
import { nextTick, onMounted, ref, watch } from 'vue';
import type { QField } from 'quasar';
import type { ConversationResource } from 'src/types/scope';

export type DraftMention = ConversationResource & { start: number; end: number };
const props = defineProps<{
  modelValue: string;
  mentions: DraftMention[];
  label: string;
  disabled: boolean;
  expanded: boolean;
  controls?: string | undefined;
  rules: ((value: unknown) => boolean | string)[];
}>();
const emit = defineEmits<{
  'update:modelValue': [string];
  'update:mentions': [DraftMention[]];
  keydown: [KeyboardEvent];
  keyup: [KeyboardEvent];
  cursorChange: [];
}>();
const editor = ref<HTMLElement | null>(null),
  field = ref<QField | null>(null);
const tokens = new Map<string, ConversationResource>();
let tokenId = 0;
type Segment = {
  node: Node;
  text: string;
  resource?: ConversationResource | undefined;
  start: number;
  end: number;
};
function segments(root: Node = editor.value!): Segment[] {
  const result: Segment[] = [];
  let offset = 0;
  function add(node: Node, text: string, resource?: ConversationResource) {
    result.push({ node, text, resource, start: offset, end: offset + text.length });
    offset += text.length;
  }
  function walk(node: Node) {
    if (node.nodeType === Node.TEXT_NODE) {
      add(node, node.textContent ?? '');
      return;
    }
    if (!(node instanceof HTMLElement)) return;
    const resource = tokens.get(node.dataset.mentionKey ?? '');
    if (resource) {
      add(node, `${resource.kind === 'file' ? '#' : '@'}${resource.label}`, resource);
      return;
    }
    if (node.tagName === 'BR') {
      if (node !== root.lastChild || offset > 0) add(node, '\n');
      return;
    }
    if (
      node !== root &&
      ['DIV', 'P'].includes(node.tagName) &&
      offset &&
      !result.at(-1)?.text.endsWith('\n')
    )
      add(node, '\n');
    node.childNodes.forEach(walk);
  }
  root.childNodes.forEach(walk);
  return result;
}
function state() {
  const parts = segments();
  return {
    text: parts.map((p) => p.text).join(''),
    mentions: parts.flatMap((p) =>
      p.resource ? [{ ...p.resource, start: p.start, end: p.end }] : [],
    ),
  };
}
function cursor() {
  const selection = window.getSelection(),
    el = editor.value;
  if (!selection?.rangeCount || !el) return null;
  const selected = selection.getRangeAt(0);
  if (!el.contains(selected.startContainer) || !el.contains(selected.endContainer)) return null;
  const before = selected.cloneRange();
  before.selectNodeContents(el);
  before.setEnd(selected.startContainer, selected.startOffset);
  const through = selected.cloneRange();
  through.selectNodeContents(el);
  through.setEnd(selected.endContainer, selected.endOffset);
  return {
    start: segments(before.cloneContents()).reduce((n, p) => n + p.text.length, 0),
    end: segments(through.cloneContents()).reduce((n, p) => n + p.text.length, 0),
  };
}
function setPoint(range: Range, offset: number, end: boolean) {
  const el = editor.value!;
  for (const part of segments()) {
    if (offset > part.end) continue;
    const set = end ? 'setEnd' : 'setStart';
    if (part.node.nodeType === Node.TEXT_NODE)
      range[set](part.node, Math.max(0, offset - part.start));
    else if (offset <= part.start) range[end ? 'setEndBefore' : 'setStartBefore'](part.node);
    else range[end ? 'setEndAfter' : 'setStartAfter'](part.node);
    return;
  }
  range[end ? 'setEnd' : 'setStart'](el, el.childNodes.length);
}
function select(start: number, end = start) {
  const range = document.createRange();
  setPoint(range, start, false);
  setPoint(range, end, true);
  const selection = window.getSelection();
  selection?.removeAllRanges();
  selection?.addRange(range);
  return range;
}
function expandedRange(start: number, end: number) {
  for (const mention of state().mentions)
    if (start < mention.end && end > mention.start) {
      start = Math.min(start, mention.start);
      end = Math.max(end, mention.end);
    }
  return { start, end };
}
function sync() {
  const value = state();
  emit('update:modelValue', value.text);
  emit('update:mentions', value.mentions);
  void nextTick().then(() => emit('cursorChange'));
}
function insertText(value: string) {
  const current = cursor();
  if (!current) return;
  const bounds = expandedRange(current.start, current.end);
  if (state().text.length - (bounds.end - bounds.start) + value.length > 4000) return;
  const range = select(bounds.start, bounds.end);
  range.deleteContents();
  range.insertNode(document.createTextNode(value));
  select(bounds.start + value.length);
  sync();
}
function chip(resource: ConversationResource) {
  const node = document.createElement('span'),
    key = String(++tokenId);
  tokens.set(key, resource);
  node.dataset.mentionKey = key;
  node.dataset.mentionKind = resource.kind;
  node.className = 'conversation-mention-chip';
  node.contentEditable = 'false';
  node.spellcheck = false;
  node.textContent = `${resource.kind === 'file' ? '#' : '@'}${resource.label}`;
  return node;
}
function insertMention(resource: ConversationResource, start: number, end: number) {
  if (!editor.value) return;
  const token = `${resource.kind === 'file' ? '#' : '@'}${resource.label}`;
  if (state().text.length - (end - start) + token.length + 1 > 4000) return;
  const range = select(start, end);
  range.deleteContents();
  const fragment = document.createDocumentFragment();
  fragment.append(chip(resource), document.createTextNode(' '));
  range.insertNode(fragment);
  editor.value.focus();
  select(start + token.length + 1);
  sync();
}
function atomicDelete(backward: boolean) {
  const current = cursor();
  if (!current) return false;
  let { start, end } = current;
  if (start === end) {
    const mention = state().mentions.find((m) => (backward ? m.end === start : m.start === start));
    if (!mention) return false;
    start = mention.start;
    end = mention.end;
  } else {
    const bounds = expandedRange(start, end);
    if (bounds.start === start && bounds.end === end) return false;
    start = bounds.start;
    end = bounds.end;
  }
  select(start, end).deleteContents();
  select(start);
  sync();
  return true;
}
function beforeInput(event: InputEvent) {
  if (event.isComposing) return;
  const selection = cursor();
  if (
    event.inputType.startsWith('insert') &&
    event.data &&
    selection &&
    state().text.length - (selection.end - selection.start) + event.data.length > 4000
  ) {
    event.preventDefault();
    return;
  }
  if (event.inputType === 'deleteContentBackward' || event.inputType === 'deleteContentForward') {
    if (atomicDelete(event.inputType === 'deleteContentBackward')) event.preventDefault();
  }
}
function keydown(event: KeyboardEvent) {
  if (props.disabled) {
    event.preventDefault();
    return;
  }
  emit('keydown', event);
  if (event.defaultPrevented || event.isComposing || event.keyCode === 229) return;
  if (
    (event.key === 'Backspace' || event.key === 'Delete') &&
    atomicDelete(event.key === 'Backspace')
  )
    event.preventDefault();
  if (event.key === 'Enter' && event.shiftKey) {
    event.preventDefault();
    insertText('\n');
  }
}
function paste(event: ClipboardEvent) {
  event.preventDefault();
  insertText(event.clipboardData?.getData('text/plain') ?? '');
}
function render() {
  const el = editor.value;
  if (!el) return;
  const current = state();
  if (
    current.text === props.modelValue &&
    JSON.stringify(current.mentions) === JSON.stringify(props.mentions)
  )
    return;
  const position = document.activeElement === el ? cursor() : null;
  el.replaceChildren();
  tokens.clear();
  let offset = 0;
  for (const mention of [...props.mentions].sort((a, b) => a.start - b.start)) {
    const token = `${mention.kind === 'file' ? '#' : '@'}${mention.label}`;
    if (mention.start < offset || props.modelValue.slice(mention.start, mention.end) !== token)
      continue;
    el.append(
      document.createTextNode(props.modelValue.slice(offset, mention.start)),
      chip(mention),
    );
    offset = mention.end;
  }
  el.append(document.createTextNode(props.modelValue.slice(offset)));
  if (position)
    select(
      Math.min(position.start, props.modelValue.length),
      Math.min(position.end, props.modelValue.length),
    );
}
watch(() => [props.modelValue, props.mentions], render, { deep: true, flush: 'post' });
onMounted(render);
defineExpose({
  focus: () => editor.value?.focus(),
  cursor,
  insertMention,
  validate: () => field.value?.validate(),
  resetValidation: () => field.value?.resetValidation(),
});
</script>
<style scoped>
.conversation-message-editor {
  min-height: 28px;
  max-height: 180px;
  overflow-y: auto;
  width: 100%;
  white-space: pre-wrap;
  overflow-wrap: anywhere;
  outline: none;
  padding: 4px 0;
}
.conversation-message-editor:empty::before {
  content: attr(data-placeholder);
  color: var(--text-muted);
  pointer-events: none;
}
.conversation-message-editor :deep(.conversation-mention-chip) {
  display: inline-block;
  padding: 0 6px;
  border-radius: var(--radius-control);
  background: var(--surface-raised);
  border: 1px solid var(--border-subtle);
  color: var(--text-accent);
  font-weight: 600;
  white-space: pre-wrap;
  user-select: all;
}
.conversation-message-field :deep(.q-field__bottom) {
  position: static;
  transform: none;
  min-height: 24px;
  line-height: 1.4;
}
.conversation-message-field :deep(.q-field__control) {
  height: auto;
  min-height: 56px;
}
.conversation-message-field {
  padding-bottom: 0;
}
</style>
