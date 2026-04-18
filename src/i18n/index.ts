import i18n from 'i18next';
import { initReactI18next } from 'react-i18next';
import zhCN from './locales/zh-CN.json';
import enUS from './locales/en-US.json';

const resources = {
  'zh-CN': { translation: zhCN },
  'en-US': { translation: enUS },
};

i18n
  .use(initReactI18next)
  .init({
    resources,
    lng: 'zh-CN',
    fallbackLng: 'zh-CN',
    interpolation: {
      escapeValue: false,
    },
  });

if (typeof window !== 'undefined' && (window as any).__TAURI_INTERNALS__) {
  import('@tauri-apps/api/core').then(({ invoke }) => {
    invoke<{ key: string; value: string }[]>('get_settings')
      .then((settings) => {
        const locale = settings.find(s => s.key === 'locale');
        if (locale && locale.value !== 'auto') {
          i18n.changeLanguage(locale.value);
        }
      })
      .catch(() => {});
  });
}

export default i18n;
