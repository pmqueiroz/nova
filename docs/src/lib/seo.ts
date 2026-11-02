import { LATEST, REPO } from './nav';
import { version } from './release';

export const softwareApp = (site: URL, description: string) => ({
  '@type': 'SoftwareApplication',
  name: 'Nova',
  description,
  url: site.href,
  image: new URL('/og.png', site).href,
  applicationCategory: 'DeveloperApplication',
  applicationSubCategory: 'Terminal emulator',
  operatingSystem: 'Windows, macOS, Linux',
  ...(version && { softwareVersion: version }),
  downloadUrl: LATEST,
  codeRepository: REPO,
  license: `${REPO}/blob/main/LICENSE`,
  isAccessibleForFree: true,
  offers: { '@type': 'Offer', price: '0', priceCurrency: 'USD' },
  author: { '@type': 'Person', name: 'pmqueiroz', url: 'https://github.com/pmqueiroz' },
});

export const breadcrumbs = (site: URL, items: { name: string; path: string }[]) => ({
  '@type': 'BreadcrumbList',
  itemListElement: items.map((item, i) => ({
    '@type': 'ListItem',
    position: i + 1,
    name: item.name,
    item: new URL(item.path, site).href,
  })),
});

export const techArticle = (url: URL, headline: string, description: string) => ({
  '@type': 'TechArticle',
  headline,
  description,
  url: url.href,
  inLanguage: 'en',
  isPartOf: { '@type': 'WebSite', name: 'Nova', url: new URL('/', url).href },
  about: { '@type': 'SoftwareApplication', name: 'Nova' },
});
