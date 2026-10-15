import type { ConversationResource } from 'src/types/scope';

export interface MessagePart {
  text: string;
  href?: string;
  resource?: ConversationResource;
}
// Enlaces HTTP(S) escapados por Vue; nunca se interpreta HTML de un mensaje.
export function messageLinks(text: string): MessagePart[] {
  const parts: MessagePart[] = [];
  let offset = 0;
  for (const match of text.matchAll(/https?:\/\/[^\s<>]+/gi)) {
    const start = match.index;
    let urlText = match[0].replace(/[.,;:!?]+$/, '');
    for (const [opening, closing] of [
      ['(', ')'],
      ['[', ']'],
      ['{', '}'],
    ]) {
      if (!opening || !closing) continue;
      while (
        urlText.endsWith(closing) &&
        urlText.split(closing).length > urlText.split(opening).length
      )
        urlText = urlText.slice(0, -1);
    }
    if (start > offset) parts.push({ text: text.slice(offset, start) });
    try {
      const url = new URL(urlText);
      parts.push({ text: urlText, href: url.href });
      offset = start + urlText.length;
    } catch {
      parts.push({ text: match[0] });
      offset = start + match[0].length;
    }
  }
  if (offset < text.length) parts.push({ text: text.slice(offset) });
  return parts;
}

// Positions from the API count Unicode characters, including emoji as one character.
export function conversationMessageParts(
  text: string,
  mentions: ConversationResource[] = [],
): MessagePart[] {
  const characters = Array.from(text);
  const parts: MessagePart[] = [];
  let offset = 0;
  const positioned = mentions
    .filter((m) => Number.isInteger(m.start) && Number.isInteger(m.end))
    .sort((a, b) => a.start! - b.start!);
  for (const mention of positioned) {
    const start = mention.start!,
      end = mention.end!;
    if (
      start < offset ||
      end <= start ||
      end > characters.length ||
      characters.slice(start, end).join('') !==
        `${mention.kind === 'file' ? '#' : '@'}${mention.label}`
    )
      continue;
    parts.push(...messageLinks(characters.slice(offset, start).join('')));
    parts.push({
      text: `${mention.kind === 'file' ? '#' : '@'}${mention.label}`,
      resource: mention,
    });
    offset = end;
  }
  parts.push(...messageLinks(characters.slice(offset).join('')));
  // Preserve references sent before inline mentions existed, inside the message body.
  for (const mention of mentions.filter((m) => m.start == null && m.end == null)) {
    parts.push(
      { text: ' ' },
      { text: `${mention.kind === 'file' ? '#' : '@'}${mention.label}`, resource: mention },
    );
  }
  return parts;
}
