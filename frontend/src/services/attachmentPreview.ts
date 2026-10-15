import { Dialog } from 'quasar';
import AttachmentPreviewDialog from 'src/components/AttachmentPreviewDialog.vue';
import {
  getPreviewAttachment,
  previewKind,
  registerPreviewAttachments,
  type PreviewAttachment,
} from 'src/utils/attachmentPreview';

export function openAttachmentPreview(attachment: PreviewAttachment) {
  registerPreviewAttachments([attachment]);
  const item = getPreviewAttachment(attachment.id);
  if (previewKind(item.original_name, item.mime_type) === 'pdf') {
    window.open(`/api/v1/attachments/${item.id}/raw`, '_blank', 'noopener');
    return;
  }
  if (previewKind(item.original_name, item.mime_type) === 'unsupported') {
    const link = document.createElement('a');
    link.href = `/api/v1/attachments/${item.id}/raw`;
    link.download = item.original_name ?? '';
    link.click();
    return;
  }
  Dialog.create({ component: AttachmentPreviewDialog, componentProps: { attachment: item } });
}
