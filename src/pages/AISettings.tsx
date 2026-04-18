import React, { useState } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { useTranslation } from 'react-i18next';
import { Plus, Edit, Trash2, CheckCircle, XCircle, Zap, Bot } from 'lucide-react';
import ConfirmDialog from '../components/ConfirmDialog';
import {
  getAIProviders,
  getAllAIProfiles,
  getActiveAIProfile,
  addAIProfile,
  updateAIProfile,
  deleteAIProfile,
  setActiveAIProfile,
  testAIProfile,
  AIProfile,
  AddAIProfileRequest,
  UpdateAIProfileRequest,
} from '../tauri-api';

const AISettings: React.FC = () => {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const [showForm, setShowForm] = useState(false);
  const [editingId, setEditingId] = useState<string | null>(null);
  const [formData, setFormData] = useState<Partial<AddAIProfileRequest>>({
    name: '',
    provider: 'openai',
    model: '',
    apiKey: '',
  });
  const [testing, setTesting] = useState(false);
  const [testResult, setTestResult] = useState<{ success: boolean; message: string } | null>(null);
  const [deleteConfirm, setDeleteConfirm] = useState<{ isOpen: boolean; profileId: string | null; profileName: string }>({
    isOpen: false,
    profileId: null,
    profileName: '',
  });
  const [saveSuccess, setSaveSuccess] = useState(false);

  const { data: providers = [] } = useQuery({
    queryKey: ['aiProviders'],
    queryFn: getAIProviders,
  });

  const { data: profiles = [] } = useQuery({
    queryKey: ['aiProfiles'],
    queryFn: getAllAIProfiles,
  });

  const { data: _activeProfile } = useQuery({
    queryKey: ['activeAIProfile'],
    queryFn: getActiveAIProfile,
  });

  const addMutation = useMutation({
    mutationFn: addAIProfile,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['aiProfiles'] });
      queryClient.invalidateQueries({ queryKey: ['activeAIProfile'] });
      setShowForm(false);
      setFormData({ name: '', provider: 'openai', model: '', apiKey: '' });
      setSaveSuccess(true);
      setTimeout(() => setSaveSuccess(false), 3000);
    },
    onError: (error: Error) => {
      alert(t('common.saveFailed') + ': ' + error.message);
    },
  });

  const updateMutation = useMutation({
    mutationFn: ({ id, data }: { id: string; data: UpdateAIProfileRequest }) =>
      updateAIProfile(id, data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['aiProfiles'] });
      setEditingId(null);
      setFormData({ name: '', provider: 'openai', model: '', apiKey: '' });
      setSaveSuccess(true);
      setTimeout(() => setSaveSuccess(false), 3000);
    },
    onError: (error: Error) => {
      alert(t('common.saveFailed') + ': ' + error.message);
    },
  });

  const deleteMutation = useMutation({
    mutationFn: deleteAIProfile,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['aiProfiles'] });
      queryClient.invalidateQueries({ queryKey: ['activeAIProfile'] });
      setDeleteConfirm({ isOpen: false, profileId: null, profileName: '' });
    },
  });

  const setActiveMutation = useMutation({
    mutationFn: setActiveAIProfile,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['aiProfiles'] });
      queryClient.invalidateQueries({ queryKey: ['activeAIProfile'] });
    },
    onError: (error: Error) => {
      alert(t('common.saveFailed') + ': ' + error.message);
    },
  });

  const handleTest = async () => {
    if (!formData.name || !formData.provider || !formData.model || !formData.apiKey) return;
    setTesting(true);
    setTestResult(null);
    try {
      await testAIProfile(formData as AddAIProfileRequest);
      setTestResult({ success: true, message: t('ai.testSuccess') });
    } catch (e) {
      setTestResult({ success: false, message: t('ai.testFailed') + ': ' + e });
    } finally {
      setTesting(false);
    }
  };

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (editingId) {
      updateMutation.mutate({ id: editingId, data: formData as UpdateAIProfileRequest });
    } else {
      addMutation.mutate(formData as AddAIProfileRequest);
    }
  };

  const handleEdit = (profile: AIProfile) => {
    setEditingId(profile.id);
    setFormData({
      name: profile.name,
      provider: profile.provider,
      model: profile.model,
      baseUrl: profile.baseUrl,
      customName: profile.customName,
    });
    setTestResult(null);
  };

  const handleCancel = () => {
    setShowForm(false);
    setEditingId(null);
    setFormData({ name: '', provider: 'openai', model: '', apiKey: '' });
    setTestResult(null);
  };

  const handleDeleteClick = (profile: AIProfile) => {
    setDeleteConfirm({
      isOpen: true,
      profileId: profile.id,
      profileName: profile.name,
    });
  };

  const handleConfirmDelete = () => {
    if (deleteConfirm.profileId) {
      deleteMutation.mutate(deleteConfirm.profileId);
    }
  };

  const handleSetActive = (profile: AIProfile) => {
    if (!profile.isActive) {
      setActiveMutation.mutate(profile.id);
    }
  };

  const currentProviderInfo = providers.find(([key]) => key === formData.provider)?.[1];

  const getProviderName = (provider: string) => {
    const info = providers.find(([key]) => key === provider)?.[1];
    return info?.name || provider;
  };

  return (
    <div className="max-w-2xl mx-auto">
      <div className="flex justify-between items-center mb-6">
        <h1 className="text-2xl font-bold text-gray-900">{t('settings.ai')}</h1>
        <button
          onClick={() => setShowForm(true)}
          className="flex items-center px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700"
        >
          <Plus className="w-4 h-4 mr-2" />
          {t('common.add')}
        </button>
      </div>

      {saveSuccess && (
        <div className="mb-4 p-3 bg-green-50 border border-green-200 rounded-lg flex items-center text-green-700">
          <CheckCircle className="w-5 h-5 mr-2" />
          {t('common.saveSuccess')}
        </div>
      )}

      {(showForm || editingId) && (
        <div className="bg-white border border-gray-200 rounded-lg p-6 mb-6">
          <form onSubmit={handleSubmit}>
            <div className="space-y-4">
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">{t('ai.profileName') || '配置名称'}</label>
                <input
                  type="text"
                  required
                  value={formData.name}
                  onChange={(e) => setFormData({ ...formData, name: e.target.value })}
                  className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                  placeholder="例如：工作使用"
                />
              </div>

              <div>
                <label className="block text-sm font-medium text-gray-700 mb-2">{t('ai.provider')}</label>
                <div className="grid grid-cols-2 gap-3">
                  {providers.map(([key, info]) => (
                    <button
                      key={key}
                      type="button"
                      onClick={() => {
                        setFormData({
                          ...formData,
                          provider: key as any,
                          model: info.recommendedModel,
                          baseUrl: info.baseUrl,
                        });
                        setTestResult(null);
                      }}
                      className={`p-4 border-2 rounded-lg text-left transition-colors ${
                        formData.provider === key
                          ? 'border-blue-500 bg-blue-50'
                          : 'border-gray-200 hover:border-gray-300'
                      }`}
                    >
                      <div className="font-medium text-gray-900">{info.name || key}</div>
                    </button>
                  ))}
                </div>
              </div>

              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">{t('ai.model')}</label>
                <select
                  value={formData.model}
                  onChange={(e) => {
                    setFormData({ ...formData, model: e.target.value });
                    setTestResult(null);
                  }}
                  className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                >
                  {currentProviderInfo?.models.map((model) => (
                    <option key={model.id} value={model.id}>
                      {model.name}
                    </option>
                  ))}
                </select>
              </div>

              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">
                  {t('ai.apiKey')}
                  {editingId && <span className="text-gray-400 ml-2">（{t('common.keepEmpty')}）</span>}
                </label>
                <input
                  type="password"
                  required={!editingId}
                  value={formData.apiKey}
                  onChange={(e) => {
                    setFormData({ ...formData, apiKey: e.target.value });
                    setTestResult(null);
                  }}
                  className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                />
              </div>

              {formData.provider === 'custom' && (
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-1">{t('ai.baseUrl')}</label>
                  <input
                    type="text"
                    value={formData.baseUrl || ''}
                    onChange={(e) => {
                      setFormData({ ...formData, baseUrl: e.target.value });
                      setTestResult(null);
                    }}
                    className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                    placeholder="https://api.example.com/v1"
                  />
                </div>
              )}
            </div>
            
            {testResult && (
              <div className={`mt-4 p-3 rounded-lg flex items-center ${testResult.success ? 'bg-green-50 text-green-700 border border-green-200' : 'bg-red-50 text-red-700 border border-red-200'}`}>
                {testResult.success ? <CheckCircle className="w-5 h-5 mr-2" /> : <XCircle className="w-5 h-5 mr-2" />}
                {testResult.message}
              </div>
            )}
            
            <div className="flex justify-between mt-6">
              <button
                type="button"
                onClick={handleTest}
                disabled={testing || !formData.name || !formData.model || !formData.apiKey}
                className="px-4 py-2 text-blue-600 hover:text-blue-700 disabled:opacity-50 border border-blue-600 rounded-lg hover:bg-blue-50"
              >
                {testing ? t('ai.testing') : t('common.test')}
              </button>
              <div className="space-x-3">
                <button
                  type="button"
                  onClick={handleCancel}
                  className="px-4 py-2 text-gray-600 hover:text-gray-800 border border-gray-300 rounded-lg hover:bg-gray-50"
                >
                  {t('common.cancel')}
                </button>
                <button
                  type="submit"
                  disabled={addMutation.isPending || updateMutation.isPending}
                  className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:opacity-50"
                >
                  {addMutation.isPending || updateMutation.isPending ? t('common.saving') : t('common.save')}
                </button>
              </div>
            </div>
          </form>
        </div>
      )}

      <div className="space-y-3">
        {profiles.map((profile) => (
          <div key={profile.id} className="bg-white border border-gray-200 rounded-lg p-4">
            <div className="flex items-center justify-between">
              <div className="flex items-center">
                <Bot className="w-5 h-5 text-gray-400 mr-3" />
                <div>
                  <div className="flex items-center">
                    <p className="font-medium text-gray-900">{profile.name}</p>
                    {profile.isActive && (
                      <span className="ml-2 inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-blue-100 text-blue-800">
                        <Zap className="w-3 h-3 mr-1" />
                        正在使用
                      </span>
                    )}
                  </div>
                  <p className="text-sm text-gray-500">
                    {getProviderName(profile.provider)} · {profile.model}
                  </p>
                </div>
              </div>
              <div className="flex space-x-2">
                {!profile.isActive && (
                  <button
                    onClick={() => handleSetActive(profile)}
                    className="p-2 text-blue-600 hover:text-blue-800 hover:bg-blue-50 rounded"
                    title="设为默认"
                  >
                    <Zap className="w-4 h-4" />
                  </button>
                )}
                <button
                  onClick={() => handleEdit(profile)}
                  className="p-2 text-gray-400 hover:text-gray-600"
                >
                  <Edit className="w-4 h-4" />
                </button>
                <button
                  onClick={() => handleDeleteClick(profile)}
                  className="p-2 text-gray-400 hover:text-red-600"
                >
                  <Trash2 className="w-4 h-4" />
                </button>
              </div>
            </div>
          </div>
        ))}
      </div>

      <ConfirmDialog
        isOpen={deleteConfirm.isOpen}
        title={t('confirm.title')}
        message={t('confirm.deleteAIProfile', { name: deleteConfirm.profileName }) || `确定要删除配置 "${deleteConfirm.profileName}" 吗？`}
        confirmText={t('common.delete')}
        cancelText={t('common.cancel')}
        onConfirm={handleConfirmDelete}
        onCancel={() => setDeleteConfirm({ isOpen: false, profileId: null, profileName: '' })}
        isDanger
      />
    </div>
  );
};

export default AISettings;
