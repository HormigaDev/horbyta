import { computed, type Ref } from 'vue';
import { marked } from 'marked';
import DOMPurify from 'dompurify';
import hljs from 'highlight.js/lib/core';
import javascript from 'highlight.js/lib/languages/javascript';
import typescript from 'highlight.js/lib/languages/typescript';
import rust from 'highlight.js/lib/languages/rust';
import shell from 'highlight.js/lib/languages/shell';
import bash from 'highlight.js/lib/languages/bash';
import python from 'highlight.js/lib/languages/python';

const langNames: Record<string, string> = {
  javascript: 'javascript',
  ts: 'typescript',
  typescript: 'typescript',
  rust: 'rust',
  shell: 'shell',
  sh: 'shell',
  bash: 'bash',
  python: 'python',
};

let isConfigured = false;

const configureMarkdown = () => {
  if (isConfigured) return;

  hljs.registerLanguage('javascript', javascript);
  hljs.registerLanguage('typescript', typescript);
  hljs.registerLanguage('rust', rust);
  hljs.registerLanguage('shell', shell);
  hljs.registerLanguage('bash', bash);
  hljs.registerLanguage('python', python);
  hljs.registerLanguage('js', javascript);
  hljs.registerLanguage('ts', typescript);
  hljs.registerLanguage('sh', shell);

  const renderer = new marked.Renderer();

  renderer.code = function ({ text, lang }) {
    const language = lang || '';
    const validLang = !!(language && hljs.getLanguage(language));
    const highlighted = validLang
      ? hljs.highlight(text, { language }).value
      : hljs.highlightAuto(text).value;
    const displayLang = langNames[language.toLowerCase()] || language || 'code';

    return `<div class="code-block-wrapper">
              <div class="code-block-header">
                <div class="mac-buttons">
                  <span class="mac-btn close"></span>
                  <span class="mac-btn minimize"></span>
                  <span class="mac-btn maximize"></span>
                </div>
                <span class="lang-title">${displayLang}</span>
              </div>
              <pre><code class="hljs ${language}">${highlighted}</code></pre>
            </div>`;
  };

  marked.use({
    renderer,
    gfm: true,
    breaks: true,
    extensions: [
      {
        name: 'inlineCheckbox',
        level: 'inline' as const,
        start: (src: string) => src.indexOf('['),
        tokenizer(src: string) {
          const match = src.match(/^\[([xX ])\](?!\()/);
          if (match) {
            return { type: 'inlineCheckbox', raw: match[0], checked: (match[1] as string).toLowerCase() === 'x' };
          }
          return undefined;
        },
        renderer(token: { type: string; raw: string; [key: string]: unknown }): string {
          return `<input type="checkbox"${token['checked'] ? ' checked' : ''} disabled>`;
        },
      },
    ],
  });

  isConfigured = true;
};

export const useMarkdownRendering = (
  rawContent: Ref<string>,
  objectUrls: Ref<Record<string, string>>,
) => {
  configureMarkdown();

  const markdownForPreview = computed(() => {
    return (rawContent.value || '').replace(
      /attachment:\/\/local\/([A-Za-z0-9_-]+)/g,
      (match, id: string) => objectUrls.value[id] || match,
    );
  });

  const renderedContent = computed(() => {
    const html = marked.parse(markdownForPreview.value);
    return DOMPurify.sanitize(html as string, {
      ALLOWED_TAGS: [
        'h1',
        'h2',
        'h3',
        'h4',
        'h5',
        'h6',
        'h7',
        'h8',
        'p',
        'a',
        'br',
        'hr',
        'ul',
        'ol',
        'li',
        'b',
        'strong',
        'i',
        'em',
        'del',
        'blockquote',
        'pre',
        'code',
        'table',
        'span',
        'th',
        'tr',
        'td',
        'thead',
        'tbody',
        'div',
        'img',
        'input',
      ],
      ALLOWED_ATTR: ['href', 'src', 'alt', 'title', 'class', 'target', 'rel', 'loading', 'type', 'checked', 'disabled'],
      ALLOWED_URI_REGEXP:
        /^(?:(?:(?:f|ht)tps?|mailto|tel|callto|cid|xmpp|blob):|[^a-z]|[a-z+.-]+(?:[^a-z+.-:]|$))/i,
    });
  });

  return {
    renderedContent,
  };
};
