import React from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { useTranslation } from 'react-i18next';
import { getSettings, setSetting, Setting } from '../tauri-api';

const General: React.FC = () => {
  const { t, i18n } = useTranslation();
  const queryClient = useQueryClient();

  const { data: settings = [] } = useQuery({
    queryKey: ['settings'],
    queryFn: getSettings,
  });

  const setSettingMutation = useMutation({
    mutationFn: ({ key, value }: { key: string; value: string }) => setSetting(key, value),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['settings'] });
    },
  });

  const getSettingValue = (key: string, defaultValue: string) => {
    const setting = settings.find((s: Setting) => s.key === key);
    return setting?.value ?? defaultValue;
  };

  const handleLanguageChange = (value: string) => {
    setSettingMutation.mutate({ key: 'locale', value });
    if (value !== 'auto') {
      i18n.changeLanguage(value);
    }
  };

  const handleFetchIntervalChange = (value: string) => {
    setSettingMutation.mutate({ key: 'fetchInterval', value });
  };

  const handleDefaultRemindOffsetsChange = (value: string) => {
    setSettingMutation.mutate({ key: 'defaultRemindOffsets', value });
  };

  const handleCloseActionChange = (value: string) => {
    setSettingMutation.mutate({ key: 'closeAction', value });
  };

  return (
    <div className="max-w-2xl mx-auto">
      <h1 className="text-2xl font-bold text-gray-900 mb-6">{t('settings.general')}</h1>

      <div className="bg-white border border-gray-200 rounded-lg p-6 space-y-6">
        <div>
          <label className="block text-sm font-medium text-gray-700 mb-2">{t('settings.language')}</label>
          <select
            value={getSettingValue('locale', 'auto')}
            onChange={(e) => handleLanguageChange(e.target.value)}
            className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
          >
            <option value="auto">{t('settings.language.auto')}</option>
            <option value="zh-CN">简体中文</option>
            <option value="en-US">English</option>
          </select>
        </div>

        <div>
          <label className="block text-sm font-medium text-gray-700 mb-2">拉取间隔</label>
          <select
            value={getSettingValue('fetchInterval', '120')}
            onChange={(e) => handleFetchIntervalChange(e.target.value)}
            className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
          >
            <option value="30">30 分钟</option>
            <option value="60">1 小时</option>
            <option value="120">2 小时</option>
            <option value="180">3 小时</option>
            <option value="240">4 小时</option>
          </select>
        </div>

        <div>
          <label className="block text-sm font-medium text-gray-700 mb-2">默认提醒时间（分钟，逗号分隔）</label>
          <input
            type="text"
            value={getSettingValue('defaultRemindOffsets', '1440,120')}
            onChange={(e) => handleDefaultRemindOffsetsChange(e.target.value)}
            placeholder="1440,120"
            className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
          />
          <p className="text-sm text-gray-500 mt-1">1440 = 24小时前，120 = 2小时前</p>
        </div>

        <div>
          <label className="block text-sm font-medium text-gray-700 mb-2">关闭窗口时</label>
          <select
            value={getSettingValue('closeAction', 'minimize-to-tray')}
            onChange={(e) => handleCloseActionChange(e.target.value)}
            className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
          >
            <option value="minimize-to-tray">最小化到托盘</option>
            <option value="quit">退出应用</option>
          </select>
        </div>

        <div className="pt-4 border-t border-gray-200">
          <h3 className="text-sm font-medium text-gray-700 mb-2">关于</h3>
          <p className="text-sm text-gray-500">Jiuj 0.1.0</p>
          <p className="text-sm text-gray-500 mt-1">
            有邮件来了，该做的事别忘了
          </p>
        </div>
      </div>
    </div>
  );
};

export default General;
