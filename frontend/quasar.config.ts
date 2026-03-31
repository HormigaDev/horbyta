import { defineConfig } from '#q-app/wrappers';

export default defineConfig((/* ctx */) => {
  return {
    boot: [],
    css: ['app.scss'],
    extras: ['roboto-font', 'material-icons'],

    build: {
      target: {
        browser: 'baseline-widely-available',
        node: 'node22',
      },
      typescript: {
        strict: true,
        vueShim: true,
      },
      vueRouterMode: 'hash',
      vitePlugins: [
        [
          'vite-plugin-checker',
          {
            vueTsc: true,
            eslint: {
              lintCommand: 'eslint -c ./eslint.config.js "./src*/**/*.{ts,js,mjs,cjs,vue}"',
              useFlatConfig: true,
            },
          },
          { server: false },
        ],
      ],
    },

    devServer: {
      open: true,
      port: 9000,
      proxy: {
        '/api': {
          target: 'http://127.0.0.1:8890',
          changeOrigin: true,
        },
      },
    },

    framework: {
      config: {},
      plugins: [],
    },
    animations: [],
    ssr: {
      prodPort: 3000,
      middlewares: ['render'],
      pwa: false,
    },

    pwa: {
      workboxMode: 'GenerateSW',
    },

    cordova: {},

    capacitor: {
      hideSplashscreen: true,
    },

    electron: {
      preloadScripts: ['electron-preload'],
      inspectPort: 5858,
      bundler: 'packager',

      packager: {},

      builder: {
        appId: 'frontend',
      },
    },

    bex: {
      extraScripts: [],
    },
  };
});
