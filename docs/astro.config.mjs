import { defineConfig } from 'astro/config';
import sitemap from '@astrojs/sitemap';
import remarkDirective from 'remark-directive';
import { remarkNova, rehypeNova } from './src/lib/markdown.mjs';

const moved = ['installation', 'quickstart', 'configuration', 'keybindings', 'ai', 'cli', 'internals-osc', 'development', 'troubleshooting'];

export default defineConfig({
  site: 'https://nova.pmqueiroz.dev',
  outDir: 'dist',
  trailingSlash: 'ignore',
  compressHTML: false,
  integrations: [sitemap()],
  redirects: Object.fromEntries(moved.map((slug) => [`/${slug}`, `/docs/${slug}/`])),
  markdown: {
    syntaxHighlight: false,
    remarkPlugins: [remarkDirective, remarkNova],
    rehypePlugins: [rehypeNova],
  },
});
