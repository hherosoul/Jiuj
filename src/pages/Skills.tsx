import React, { useState } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { useTranslation } from 'react-i18next';
import { useNavigate } from 'react-router-dom';
import { Plus, Edit, Trash2, Zap, Star, X, Plus as PlusIcon } from 'lucide-react';
import ConfirmDialog from '../components/ConfirmDialog';
import {
  getAllSkills,
  getActiveSkill,
  setActiveSkill,
  createSkill,
  deleteSkill,
  Skill,
} from '../tauri-api';

interface CreateFormData {
  name: string;
  description: string;
  identity: string;
  extractRules: string[];
  priorityRules: string[];
  notifyRules: string[];
  customPrompt: string;
}

const generateSkillContent = (name: string, formData: CreateFormData): string => {
  let content = `# Skill: ${name}\n\n`;

  if (formData.identity) {
    content += `## identity\n${formData.identity}\n\n`;
  } else {
    content += `## identity\n\n`;
  }

  content += `## extract-rules\n`;
  if (formData.extractRules.length > 0) {
    formData.extractRules.forEach((rule) => {
      content += `- ${rule}\n`;
    });
  }
  content += '\n';

  content += `## priority-rules\n`;
  if (formData.priorityRules.length > 0) {
    formData.priorityRules.forEach((rule) => {
      content += `- ${rule}\n`;
    });
  }
  content += '\n';

  content += `## notify-rules\n`;
  if (formData.notifyRules.length > 0) {
    formData.notifyRules.forEach((rule) => {
      content += `- ${rule}\n`;
    });
  }
  content += '\n';

  content += `## custom-prompt\n`;
  if (formData.customPrompt) {
    content += `${formData.customPrompt}\n`;
  }

  return content;
};

const Skills: React.FC = () => {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const [showCreateForm, setShowCreateForm] = useState(false);
  const [createFormData, setCreateFormData] = useState<CreateFormData>({
    name: '',
    description: '',
    identity: '',
    extractRules: [],
    priorityRules: [],
    notifyRules: [],
    customPrompt: '',
  });
  const [deleteConfirm, setDeleteConfirm] = useState<{ isOpen: boolean; skill: Skill | null }>({
    isOpen: false,
    skill: null,
  });

  const { data: skills = [] } = useQuery({
    queryKey: ['skills'],
    queryFn: getAllSkills,
  });

  const { data: activeSkill } = useQuery({
    queryKey: ['activeSkill'],
    queryFn: getActiveSkill,
  });

  const setActiveMutation = useMutation({
    mutationFn: setActiveSkill,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['skills'] });
      queryClient.invalidateQueries({ queryKey: ['activeSkill'] });
    },
  });

  const createMutation = useMutation({
    mutationFn: () => createSkill(createFormData.name, createFormData.description, generateSkillContent(createFormData.name, createFormData)),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['skills'] });
      setShowCreateForm(false);
      setCreateFormData({
        name: '',
        description: '',
        identity: '',
        extractRules: [],
        priorityRules: [],
        notifyRules: [],
        customPrompt: '',
      });
    },
    onError: (error: Error) => {
      alert(t('common.addFailed') + ': ' + error.message);
    },
  });

  const deleteMutation = useMutation({
    mutationFn: deleteSkill,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['skills'] });
      queryClient.invalidateQueries({ queryKey: ['activeSkill'] });
      setDeleteConfirm({ isOpen: false, skill: null });
    },
  });

  const handleCreate = (e: React.FormEvent) => {
    e.preventDefault();
    if (createFormData.name) {
      createMutation.mutate();
    }
  };

  const handleDeleteClick = (skill: Skill) => {
    setDeleteConfirm({ isOpen: true, skill });
  };

  const handleConfirmDelete = () => {
    if (deleteConfirm.skill) {
      deleteMutation.mutate(deleteConfirm.skill.id);
    }
  };

  const addRule = (field: 'extractRules' | 'priorityRules' | 'notifyRules') => {
    setCreateFormData(prev => ({
      ...prev,
      [field]: [...prev[field], ''],
    }));
  };

  const removeRule = (field: 'extractRules' | 'priorityRules' | 'notifyRules', index: number) => {
    setCreateFormData(prev => ({
      ...prev,
      [field]: prev[field].filter((_, i) => i !== index),
    }));
  };

  const updateRule = (field: 'extractRules' | 'priorityRules' | 'notifyRules', index: number, value: string) => {
    setCreateFormData(prev => ({
      ...prev,
      [field]: prev[field].map((rule, i) => i === index ? value : rule),
    }));
  };

  return (
    <div className="max-w-4xl mx-auto">
      <div className="flex justify-between items-center mb-6">
        <h1 className="text-2xl font-bold text-gray-900">{t('settings.skills')}</h1>
        <button
          onClick={() => setShowCreateForm(true)}
          className="flex items-center px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700"
        >
          <Plus className="w-4 h-4 mr-2" />
          {t('common.add')}
        </button>
      </div>

      {showCreateForm && (
        <div className="bg-white border border-gray-200 rounded-lg p-6 mb-6">
          <form onSubmit={handleCreate}>
            <div className="flex justify-between items-center mb-6">
              <h2 className="text-lg font-semibold text-gray-900">创建新技能</h2>
              <button
                type="button"
                onClick={() => setShowCreateForm(false)}
                className="p-1 text-gray-400 hover:text-gray-600"
              >
                <X className="w-5 h-5" />
              </button>
            </div>

            <div className="space-y-6">
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">{t('skill.name')} *</label>
                <input
                  type="text"
                  required
                  value={createFormData.name}
                  onChange={(e) => setCreateFormData(prev => ({ ...prev, name: e.target.value }))}
                  className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                />
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">{t('skill.description')}</label>
                <input
                  type="text"
                  value={createFormData.description}
                  onChange={(e) => setCreateFormData(prev => ({ ...prev, description: e.target.value }))}
                  className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                />
              </div>

              <div>
                <label className="block text-sm font-medium text-gray-700 mb-2">身份描述 (identity)</label>
                <textarea
                  value={createFormData.identity}
                  onChange={(e) => setCreateFormData(prev => ({ ...prev, identity: e.target.value }))}
                  className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                  rows={3}
                  placeholder="描述这个技能的身份和用途..."
                />
              </div>

              <div>
                <div className="flex justify-between items-center mb-2">
                  <label className="block text-sm font-medium text-gray-700">提取规则 (extract-rules)</label>
                  <button
                    type="button"
                    onClick={() => addRule('extractRules')}
                    className="flex items-center px-3 py-1 text-sm text-blue-600 hover:text-blue-800"
                  >
                    <PlusIcon className="w-4 h-4 mr-1" />
                    添加规则
                  </button>
                </div>
                <div className="space-y-2">
                  {createFormData.extractRules.map((rule, index) => (
                    <div key={index} className="flex items-center space-x-2">
                      <input
                        type="text"
                        value={rule}
                        onChange={(e) => updateRule('extractRules', index, e.target.value)}
                        className="flex-1 px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                        placeholder="输入规则..."
                      />
                      <button
                        type="button"
                        onClick={() => removeRule('extractRules', index)}
                        className="p-2 text-gray-400 hover:text-red-600"
                      >
                        <Trash2 className="w-4 h-4" />
                      </button>
                    </div>
                  ))}
                </div>
              </div>

              <div>
                <div className="flex justify-between items-center mb-2">
                  <label className="block text-sm font-medium text-gray-700">优先级规则 (priority-rules)</label>
                  <button
                    type="button"
                    onClick={() => addRule('priorityRules')}
                    className="flex items-center px-3 py-1 text-sm text-blue-600 hover:text-blue-800"
                  >
                    <PlusIcon className="w-4 h-4 mr-1" />
                    添加规则
                  </button>
                </div>
                <div className="space-y-2">
                  {createFormData.priorityRules.map((rule, index) => (
                    <div key={index} className="flex items-center space-x-2">
                      <input
                        type="text"
                        value={rule}
                        onChange={(e) => updateRule('priorityRules', index, e.target.value)}
                        className="flex-1 px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                        placeholder="输入规则..."
                      />
                      <button
                        type="button"
                        onClick={() => removeRule('priorityRules', index)}
                        className="p-2 text-gray-400 hover:text-red-600"
                      >
                        <Trash2 className="w-4 h-4" />
                      </button>
                    </div>
                  ))}
                </div>
              </div>

              <div>
                <div className="flex justify-between items-center mb-2">
                  <label className="block text-sm font-medium text-gray-700">通知规则 (notify-rules)</label>
                  <button
                    type="button"
                    onClick={() => addRule('notifyRules')}
                    className="flex items-center px-3 py-1 text-sm text-blue-600 hover:text-blue-800"
                  >
                    <PlusIcon className="w-4 h-4 mr-1" />
                    添加规则
                  </button>
                </div>
                <div className="space-y-2">
                  {createFormData.notifyRules.map((rule, index) => (
                    <div key={index} className="flex items-center space-x-2">
                      <input
                        type="text"
                        value={rule}
                        onChange={(e) => updateRule('notifyRules', index, e.target.value)}
                        className="flex-1 px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                        placeholder="输入规则..."
                      />
                      <button
                        type="button"
                        onClick={() => removeRule('notifyRules', index)}
                        className="p-2 text-gray-400 hover:text-red-600"
                      >
                        <Trash2 className="w-4 h-4" />
                      </button>
                    </div>
                  ))}
                </div>
              </div>

              <div>
                <label className="block text-sm font-medium text-gray-700 mb-2">自定义提示 (custom-prompt)</label>
                <textarea
                  value={createFormData.customPrompt}
                  onChange={(e) => setCreateFormData(prev => ({ ...prev, customPrompt: e.target.value }))}
                  className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                  rows={4}
                  placeholder="输入自定义提示词..."
                />
              </div>
            </div>

            <div className="flex justify-end space-x-3 mt-8">
              <button
                type="button"
                onClick={() => setShowCreateForm(false)}
                className="px-4 py-2 text-gray-600 hover:text-gray-800 border border-gray-300 rounded-lg hover:bg-gray-50"
              >
                {t('common.cancel')}
              </button>
              <button
                type="submit"
                disabled={createMutation.isPending}
                className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:opacity-50"
              >
                {createMutation.isPending ? t('common.adding') : t('common.add')}
              </button>
            </div>
          </form>
        </div>
      )}

      <div className="space-y-3">
        {skills.map((skill) => (
          <div
            key={skill.id}
            className={`bg-white border border-gray-200 rounded-lg p-4 ${
              activeSkill?.id === skill.id ? 'ring-2 ring-blue-500' : ''
            }`}
          >
            <div className="flex items-center justify-between">
              <div className="flex items-center">
                <Zap className="w-5 h-5 text-gray-400 mr-3" />
                <div>
                  <div className="flex items-center">
                    <p className="font-medium text-gray-900">{skill.name}</p>
                    {activeSkill?.id === skill.id && (
                      <span className="ml-2 inline-flex items-center px-2 py-0.5 text-xs font-medium text-blue-700 bg-blue-100 rounded-full">
                        <Star className="w-3 h-3 mr-1" />
                        {t('skill.active')}
                      </span>
                    )}
                    {skill.isBuiltin && (
                      <span className="ml-2 text-xs text-gray-400">{t('skill.builtin')}</span>
                    )}
                  </div>
                  <p className="text-sm text-gray-500">{skill.description}</p>
                </div>
              </div>
              <div className="flex items-center space-x-2">
                {activeSkill?.id !== skill.id && (
                  <button
                    onClick={() => setActiveMutation.mutate(skill.id)}
                    className="px-3 py-1 text-sm text-blue-600 hover:text-blue-800"
                  >
                    {t('skill.activate')}
                  </button>
                )}
                <button
                  onClick={() => navigate(`/skills/${skill.id}`)}
                  className="p-2 text-gray-400 hover:text-gray-600"
                >
                  <Edit className="w-4 h-4" />
                </button>
                {!skill.isBuiltin && (
                  <button
                    onClick={() => handleDeleteClick(skill)}
                    className="p-2 text-gray-400 hover:text-red-600"
                  >
                    <Trash2 className="w-4 h-4" />
                  </button>
                )}
              </div>
            </div>
          </div>
        ))}
      </div>

      <ConfirmDialog
        isOpen={deleteConfirm.isOpen}
        title={t('confirm.title')}
        message={deleteConfirm.skill ? t('confirm.deleteSkill', { name: deleteConfirm.skill.name }) : ''}
        confirmText={t('common.delete')}
        cancelText={t('common.cancel')}
        onConfirm={handleConfirmDelete}
        onCancel={() => setDeleteConfirm({ isOpen: false, skill: null })}
        isDanger
      />
    </div>
  );
};

export default Skills;
