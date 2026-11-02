import { visit, SKIP } from 'unist-util-visit';
import { toString } from 'mdast-util-to-string';
import GithubSlugger from 'github-slugger';
import { icons } from './icons.ts';
import { OSES, OS_ALIASES } from './os.ts';

const esc = (s) => s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');
const svg = (name) => {
  const [attrs, inner] = icons[name];
  return `<svg aria-hidden="true" ${attrs}>${inner}</svg>`;
};
const html = (value) => ({ type: 'html', value });
const el = (hName, className, children) => ({
  type: 'novaElement',
  data: { hName, hProperties: className ? { className } : {} },
  children,
});

const copyBtn = (label = true) => `<button class="copy-btn" data-copy="">${svg('copy')}${label ? '<span>Copy</span>' : ''}</button>`;

const titleFromMeta = (meta) => meta?.match(/title="([^"]+)"/)?.[1];

const codeBlock = (node) => {
  const lang = node.lang || 'text';
  const head = titleFromMeta(node.meta) || lang;
  return `<div class="code"><div class="code-head"><span>${esc(head)}</span>${copyBtn()}</div><pre><code data-lang="${esc(lang)}">${esc(node.value)}</code></pre></div>`;
};

const cmdline = (node) => {
  const lines = node.value.split('\n').map((l) => `<div>${esc(l)}</div>`).join('');
  return `<div class="cmdline"><div class="lines">${lines}</div>${copyBtn()}</div>`;
};

const CALLOUTS = { note: ['note', 'note'], tip: ['tip', 'tip'], warning: ['warn', 'warn'], warn: ['warn', 'warn'] };

const keysHtml = (combo) =>
  `<span class="keys">${combo
    .split('+')
    .map((k) => k.trim())
    .map((k) =>
      k === 'Mod' ? '<span class="kbd" data-mod="">⌘</span>' : k === 'Alt' ? '<span class="kbd" data-alt="">⌥</span>' : `<span class="kbd">${esc(k)}</span>`,
    )
    .join('')}</span>`;

const callout = (node) => {
  const [cls, icon] = CALLOUTS[node.name];
  const first = node.children[0];
  const labelled = first?.data?.directiveLabel;
  const rest = labelled ? node.children.slice(1) : node.children;
  const body = labelled ? [el('b', null, first.children), ...rest] : rest;
  return el('div', ['callout', cls], [html(svg(icon)), el('div', null, body)]);
};

const osTabs = (node) => {
  const panels = node.children.filter((c) => c.type === 'containerDirective' && OS_ALIASES[c.name]);
  const seg = `<div class="seg" role="tablist">${OSES.filter((o) => panels.some((p) => OS_ALIASES[p.name] === o.id))
    .map((o) => `<button role="tab" data-os="${o.id}">${o.label}</button>`)
    .join('')}</div>`;
  const out = panels.map((p, i) => {
    const children = p.children.map((c) => (c.type === 'code' ? html(cmdline(c)) : c));
    const panel = el('div', null, children);
    panel.data.hProperties = { dataOsPanel: OS_ALIASES[p.name], hidden: i > 0 };
    return panel;
  });
  return el('div', ['os-tabs'], [html(seg), ...out]);
};

const asText = (node) => {
  const label = node.children?.length ? `[${toString(node)}]` : '';
  return { type: 'text', value: `:${node.name}${label}` };
};

export function remarkNova() {
  return (tree) => {
    visit(tree, (node, index, parent) => {
      if (!parent || index === undefined) return;
      if (node.type === 'containerDirective' && node.name === 'os-tabs') {
        parent.children[index] = osTabs(node);
        return SKIP;
      }
      if (node.type === 'containerDirective' && CALLOUTS[node.name]) {
        parent.children[index] = callout(node);
        return;
      }
      if (node.type === 'textDirective' && node.name === 'kbd') {
        parent.children[index] = html(keysHtml(toString(node)));
        return SKIP;
      }
      if (node.type === 'textDirective' || node.type === 'leafDirective') {
        parent.children[index] = asText(node);
        return SKIP;
      }
    });
    visit(tree, 'code', (node, index, parent) => {
      if (!parent || index === undefined) return;
      parent.children[index] = html(codeBlock(node));
    });
  };
}

const text = (node) =>
  node.type === 'text' ? node.value : (node.children || []).map(text).join('');

export function rehypeNova() {
  return (tree) => {
    const slugger = new GithubSlugger();
    visit(tree, 'element', (node, index, parent) => {
      if (node.tagName === 'table' && parent && index !== undefined && !parent.properties?.className?.includes?.('table-wrap')) {
        parent.children[index] = { type: 'element', tagName: 'div', properties: { className: ['table-wrap'] }, children: [node] };
        return SKIP;
      }
      if (node.tagName === 'h2' || node.tagName === 'h3') {
        const id = node.properties.id || slugger.slug(text(node));
        node.properties.id = id;
        node.children.unshift({ type: 'element', tagName: 'a', properties: { className: ['anchor'], href: `#${id}` }, children: [{ type: 'text', value: '#' }] });
      }
    });
  };
}
