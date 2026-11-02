export const REPO = 'https://github.com/pmqueiroz/nova';
export const RELEASES = `${REPO}/releases`;
export const LATEST = `${RELEASES}/latest`;
export const SPONSORS = 'https://github.com/sponsors/pmqueiroz';
export const DOCS_SRC = 'docs/src/content/docs';

export interface NavPage {
  slug: string;
  title: string;
}

export interface NavGroup {
  title: string;
  pages: NavPage[];
}

export const groups: NavGroup[] = [
  {
    title: 'Getting started',
    pages: [
      { slug: 'index', title: 'Introduction' },
      { slug: 'installation', title: 'Installation' },
      { slug: 'quickstart', title: 'Quickstart' },
    ],
  },
  {
    title: 'Reference',
    pages: [
      { slug: 'configuration', title: 'Configuration' },
      { slug: 'keybindings', title: 'Keybindings' },
      { slug: 'ai', title: 'AI' },
      { slug: 'cli', title: 'CLI' },
    ],
  },
  {
    title: 'Internals',
    pages: [
      { slug: 'internals-osc', title: 'OSC integration' },
      { slug: 'development', title: 'Development' },
    ],
  },
  {
    title: 'Help',
    pages: [{ slug: 'troubleshooting', title: 'Troubleshooting' }],
  },
];

export const flat = groups.flatMap((g) => g.pages.map((p) => ({ ...p, group: g.title })));

export const docHref = (slug: string) => (slug === 'index' ? '/docs/' : `/docs/${slug}/`);

export const neighbours = (slug: string) => {
  const i = flat.findIndex((p) => p.slug === slug);
  return { prev: i > 0 ? flat[i - 1] : undefined, next: i >= 0 && i < flat.length - 1 ? flat[i + 1] : undefined };
};

export const findPage = (slug: string) => flat.find((p) => p.slug === slug);
