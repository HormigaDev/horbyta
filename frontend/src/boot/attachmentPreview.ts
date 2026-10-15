import { defineBoot } from '#q-app/wrappers';
import { openAttachmentPreview } from 'src/services/attachmentPreview';
import { attachmentIdFromUrl, getPreviewAttachment } from 'src/utils/attachmentPreview';

export default defineBoot(() => {
  // Los enlaces del Markdown sanitizado se resuelven igual que los de listas y alcance.
  document.addEventListener(
    'click',
    (event) => {
      if (!(event instanceof MouseEvent) || event.button !== 0 || event.ctrlKey || event.metaKey || event.shiftKey || event.altKey)
        return;
      const target = event.target instanceof Element ? event.target : null;
      const link = target?.closest<HTMLAnchorElement>('a[href]');
      if (link?.hasAttribute('download')) return;
      const image = target?.closest<HTMLImageElement>('.markdown-preview img');
      const id = attachmentIdFromUrl(link?.href ?? image?.src ?? '');
      if (!id) return;
      event.preventDefault();
      event.stopPropagation();
      const cached = getPreviewAttachment(id);
      openAttachmentPreview(
        image && !cached.mime_type ? { ...cached, mime_type: 'image/' } : cached,
      );
    },
    true,
  );
});
