// @ts-check
import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';
import tailwind from '@astrojs/tailwind';

// https://astro.build/config
export default defineConfig({
  integrations: [starlight({
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
