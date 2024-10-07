// @ts-check
import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';
import tailwind from '@astrojs/tailwind';

// https://astro.build/config
export default defineConfig({
  integrations: [starlight({
    head: [
      {
        tag: 'script',
        attrs: {
          src: 'https://www.googletagmanager.com/gtag/js?id=G-CYLN707PNH',
          defer: true
        }
      }
    ],
    title: 'RustyWatch',
    customCss: [
      '@fontsource/dejavu-sans/400.css',
      './src/tailwind.css',
      './src/styles/custom.css'
    ],
    social: {
      github: 'https://github.com/ak9024/rustywatch',
    },
    components: {
      Head: "./src/components/Head.astro",
      Hero: "./src/components/Hero.astro",
      Footer: "./src/components/Footer.astro"
    },
    sidebar: [
      {
        label: 'Getting Started',
        autogenerate: { directory: 'getting-started' },
      },
      {
        label: 'Installation',
        autogenerate: { directory: 'installation' },
      },
      {
        label: 'Usage',
        autogenerate: { directory: 'usage' },
      },
      {
        label: 'Reference',
        autogenerate: { directory: 'reference' },
      },
    ],
  }), tailwind({
    applyBaseStyles: false,
  })],
});
