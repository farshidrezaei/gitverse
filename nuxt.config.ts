export default defineNuxtConfig({
  ssr: false,
  telemetry: false,
  compatibilityDate: '2025-07-15',

  app: {
    head: {
      title: 'GitVerse',
      meta: [
        { charset: 'utf-8' },
        { name: 'viewport', content: 'width=device-width, initial-scale=1' },
      ],
      link: [
        { rel: 'icon', type: 'image/svg+xml', href: '/favicon.svg' },
      ],
    },
  },

  nitro: {
    preset: 'static',
  },

  devtools: {
    enabled: false,
  },

  typescript: {
    strict: true,
  },

  modules: ['@nuxt/ui', '@nuxtjs/i18n'],

  css: ['~/assets/css/main.css'],

  colorMode: {
    preference: 'dark',
    fallback: 'dark',
    storageKey: 'gitverse-color-mode',
  },

  icon: {
    clientBundle: {
      scan: {
        globInclude: ['**/*.{vue,jsx,tsx,md,mdc,mdx,yml,yaml}', 'app/**/*.{ts,mts}'],
      },
    },
  },

  i18n: {
    defaultLocale: 'en',
    strategy: 'no_prefix',
    langDir: 'locales',
    detectBrowserLanguage: false,
    locales: [
      {
        code: 'en',
        language: 'en-US',
        name: 'English',
        dir: 'ltr',
        file: 'en.json',
      },
      {
        code: 'fa',
        language: 'fa-IR',
        name: 'فارسی',
        dir: 'rtl',
        file: 'fa.json',
      },
    ],
  },
})
