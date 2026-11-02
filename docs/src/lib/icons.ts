const s16 = 'viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.4"';
const s16r = `${s16} stroke-linecap="round"`;
const s20 = 'viewBox="0 0 20 20" fill="none" stroke="currentColor" stroke-width="1.6"';

export const icons = {
  logo: [
    'viewBox="0 0 100 100"',
    '<g fill="none" stroke="currentColor" stroke-linecap="round" stroke-width="7"><line x1="50" y1="50" x2="50" y2="10"></line><line x1="50" y1="50" x2="50" y2="90"></line><line x1="50" y1="50" x2="10" y2="50"></line><line x1="50" y1="50" x2="90" y2="50"></line></g><circle cx="50" cy="50" r="13" fill="currentColor"></circle>',
  ],
  github: [
    'viewBox="0 0 16 16" fill="currentColor"',
    '<path d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27.68 0 1.36.09 2 .27 1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.013 8.013 0 0016 8c0-4.42-3.58-8-8-8z"></path>',
  ],
  sun: [
    `${s20} stroke-linecap="round"`,
    '<circle cx="10" cy="10" r="3.5"></circle><path d="M10 2v2M10 16v2M2 10h2M16 10h2M4.3 4.3l1.4 1.4M14.3 14.3l1.4 1.4M4.3 15.7l1.4-1.4M14.3 5.7l1.4-1.4"></path>',
  ],
  moon: [`${s20} stroke-linejoin="round"`, '<path d="M16 12.5A6.5 6.5 0 0 1 7.5 4a6.5 6.5 0 1 0 8.5 8.5z"></path>'],
  menu: [`${s20} stroke-linecap="round"`, '<path d="M3 6h14M3 10h14M3 14h14"></path>'],
  copy: [
    s16,
    '<rect x="5" y="5" width="9" height="9" rx="2"></rect><path d="M11 5V3.5A1.5 1.5 0 0 0 9.5 2h-6A1.5 1.5 0 0 0 2 3.5v6A1.5 1.5 0 0 0 3.5 11H5"></path>',
  ],
  plus: [s16r, '<path d="M8 3v10M3 8h10"></path>'],
  split: [s16, '<rect x="2" y="3" width="12" height="10" rx="2"></rect><path d="M8 3v10"></path>'],
  shell: [s16, '<path d="M3 4.5 6 7.5 3 10.5M8 11h5"></path>'],
  search: [s16r, '<circle cx="7" cy="7" r="4.5"></circle><path d="m10.5 10.5 3 3"></path>'],
  'search-lg': [`${s20} stroke-linecap="round"`, '<circle cx="9" cy="9" r="5.5"></circle><path d="m13 13 4 4"></path>'],
  sparkle: ['viewBox="0 0 20 20" fill="currentColor"', '<path d="M10 2l1.6 4.8L16.4 8.4 11.6 10 10 14.8 8.4 10 3.6 8.4l4.8-1.6z"></path>'],
  sparkles: [
    'viewBox="0 0 20 20" fill="currentColor"',
    '<path d="M10 2l1.6 4.8L16.4 8.4 11.6 10 10 14.8 8.4 10 3.6 8.4l4.8-1.6z"></path><path d="M15.5 13l.7 2 2 .7-2 .7-.7 2-.7-2-2-.7 2-.7z"></path>',
  ],
  warn: [
    `${s20} stroke-linejoin="round" stroke-linecap="round"`,
    '<path d="M10 3 18 17H2z"></path><path d="M10 8.5v3.5M10 14.5v.01"></path>',
  ],
  note: [`${s20} stroke-linecap="round"`, '<circle cx="10" cy="10" r="7.5"></circle><path d="M10 9v5M10 6.5v.01"></path>'],
  tip: [
    `${s20} stroke-linecap="round" stroke-linejoin="round"`,
    '<path d="M7.5 15h5M8 17.5h4M10 2.5a5 5 0 0 0-3 9c.6.5 1 1.2 1 2V14h4v-.5c0-.8.4-1.5 1-2a5 5 0 0 0-3-9z"></path>',
  ],
  download: [
    'viewBox="0 0 12 12" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round"',
    '<path d="M6 2v7M3 6l3 3 3-3"></path>',
  ],
  heart: [
    'viewBox="0 0 20 20" fill="currentColor"',
    '<path d="M10 17s-7-4.35-7-9.5A3.5 3.5 0 0 1 10 5.5a3.5 3.5 0 0 1 7 2C17 12.65 10 17 10 17z"></path>',
  ],
  star: [`${s16} stroke-linejoin="round"`, '<path d="m8 1.5 2 4.2 4.5.6-3.3 3.1.8 4.5L8 11.7l-4 2.2.8-4.5L1.5 6.3 6 5.7z"></path>'],
  issue: [s16r, '<circle cx="8" cy="8" r="6"></circle><path d="M8 5v3.5M8 11v.01"></path>'],
  code: [`${s16r} stroke-linejoin="round"`, '<path d="M5.5 4 2 8l3.5 4M10.5 4 14 8l-3.5 4"></path>'],
  edit: [`${s16r} stroke-linejoin="round"`, '<path d="M10.5 2.5 13.5 5.5 6 13H3v-3z"></path>'],
  sidebar: [s16r, '<rect x="2" y="2.5" width="12" height="11" rx="2"></rect><path d="M6 2.5v11"></path>'],
} satisfies Record<string, [string, string]>;

export type IconName = keyof typeof icons;
