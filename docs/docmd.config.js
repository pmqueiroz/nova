export default defineConfig({
  title: 'A terminal that burns bright',
  url: 'https://nova.pmqueiroz.dev',
  logo: {
    light: 'assets/images/nova-logo-dark.png',
    dark: 'assets/images/nova-logo-light.png',
    alt: 'Nova Logo',
    href: '/',
  },
  favicon: 'assets/favicon.ico',
  src: 'content',
  out: 'dist',
  layout: {
    spa: true,
    header: {
      enabled: true,
    },
    sidebar: {
      collapsible: true,
      defaultCollapsed: false,
    },
    optionsMenu: {
      position: 'sidebar-top',
      components: {
        search: true,      
        themeSwitch: true, 
        sponsor: null,     
      }
    },
    footer: {
      style: 'minimal',
      content: '© ' + new Date().getFullYear() + ' Nova.',
      branding: true
    }
  },
  theme: {
    name: 'default',
    appearance: 'dark',
    codeHighlight: true,    
    customCss: [],          
  },
  minify: true,           
  autoTitleFromH1: true,  
  copyCode: true,         
  pageNavigation: true,   
  customJs: [],
  navigation: [
    { title: 'Introduction', path: '/', icon: 'home' },
    {
      title: 'Getting Started',
      icon: 'rocket',
      collapsible: false,
      children: [
        { title: 'Installation', path: '/installation', icon: 'download' },
        { title: 'Configuration', path: '/configuration', icon: 'cog' },
      ],
    },
    { title: 'GitHub', path: 'https://github.com/pmqueiroz/nova', icon: 'github', external: true },
  ],
  plugins: {
    seo: {
      defaultDescription: 'A terminal that burns bright',
      openGraph: { defaultImage: '' },
      twitter: { cardType: 'summary_large_image' }
    },
    sitemap: { defaultChangefreq: 'weekly' },
    analytics: { 
      googleV4: { measurementId: 'G-X9WTDL262N' }
    }
  },
  editLink: {
    enabled: false,
    baseUrl: 'https://github.com/pmqueiroz/nova/edit/main/docs',
    text: 'Edit this page'
  }
});
