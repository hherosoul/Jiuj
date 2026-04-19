import React, { useState } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { useTranslation } from 'react-i18next';
import { useParams, useNavigate } from 'react-router-dom';
import { ArrowLeft, Save, RotateCcw, CheckCircle, Code, Type, Plus, Trash2 } from 'lucide-react';
import {
  getSkillById,
  getSkillContent,
  saveSkillContent,
} from '../tauri-api';

interface FormData {
  identity: string;
  extractRules: string[];
  priorityRules: string[];
  notifyRules: string[];
  customPrompt: string;
}

const parseSkillContent = (content: string): FormData => {
  const result: FormData = {
    identity: '',
    extractRules: [],
    priorityRules: [],
    notifyRules: [],
    customPrompt: '',
  };

  const sections = content.split(/^## /gm);
  sections.forEach((section) => {
    const lines = section.trim().split('\n');
    const sectionName = lines[0].trim();
    const sectionContent = lines.slice(1).join('\n').trim();

    if (sectionName === 'identity') {
      result.identity = sectionContent;
    } else if (sectionName === 'extract-rules') {
      result.extractRules = sectionContent
        .split('\n')
        .filter((line) => line.trim().startsWith('- '))
        .map((line) => line.trim().replace(/^-\s*/, ''));
    } else if (sectionName === 'priority-rules') {
      result.priorityRules = sectionContent
        .split('\n')
        .filter((line) => line.trim().startsWith('- '))
        .map((line) => line.trim().replace(/^-\s*/, ''));
    } else if (sectionName === 'notify-rules') {
      result.notifyRules = sectionContent
        .split('\n')
        .filter((line) => line.trim().startsWith('- '))
        .map((line) => line.trim().replace(/^-\s*/, ''));
    } else if (sectionName === 'custom-prompt') {
      result.customPrompt = sectionContent;
    }
  });

  return result;
};

const generateSkillContent = (skillName: string, formData: FormData): string => {
  let content = `# Skill: ${skillName}\n\n`;

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

const SkillEditor: React.FC = () => {
  const { t } = useTranslation();
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const [content, setContent] = useState('');
  const [hasChanges, setHasChanges] = useState(false);
  const [saveSuccess, setSaveSuccess] = useState(false);
  const [isFormMode, setIsFormMode] = useState(true);
  const [formData, setFormData] = useState<FormData>({
    identity: '',
    extractRules: [],
    priorityRules: [],
    notifyRules: [],
    customPrompt: '',
  });

  const { data: skill } = useQuery({
    queryKey: ['skill', id],
    queryFn: () => getSkillById(id!),
    enabled: !!id,
  });

  const { data: skillContent } = useQuery({
    queryKey: ['skillContent', id],
    queryFn: () => getSkillContent(id!),
    enabled: !!id,
  });

  React.useEffect(() => {
    if (skillContent) {
      setContent(skillContent);
      setFormData(parseSkillContent(skillContent));
      setHasChanges(false);
    }
  }, [skillContent]);

  const saveMutation = useMutation({
    mutationFn: (newContent: string) => saveSkillContent(id!, newContent),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['skillContent', id] });
      setHasChanges(false);
      setSaveSuccess(true);
      setTimeout(() => setSaveSuccess(false), 3000);
    },
    onError: (error: Error) => {
      alert(t('common.saveFailed') + ': ' + error.message);
    },
  });

  const handleContentChange = (newContent: string) => {
    setContent(newContent);
    setHasChanges(newContent !== skillContent);
    setSaveSuccess(false);
  };

  const handleFormChange = (newFormData: FormData) => {
    setFormData(newFormData);
    const newContent = generateSkillContent(skill?.name || '', newFormData);
    setContent(newContent);
    setHasChanges(newContent !== skillContent);
    setSaveSuccess(false);
  };

  const handleSave = () => {
    saveMutation.mutate(content);
  };

  const handleReset = () => {
    if (skillContent) {
      setContent(skillContent);
      setFormData(parseSkillContent(skillContent));
      setHasChanges(false);
      setSaveSuccess(false);
    }
  };

  const handleToggleMode = () => {
    if (isFormMode) {
      setIsFormMode(false);
    } else {
      setFormData(parseSkillContent(content));
      setIsFormMode(true);
    }
  };

  const addRule = (field: 'extractRules' | 'priorityRules' | 'notifyRules') => {
    const newFormData = { ...formData };
    newFormData[field] = [...newFormData[field], ''];
    handleFormChange(newFormData);
  };

  const removeRule = (field: 'extractRules' | 'priorityRules' | 'notifyRules', index: number) => {
    const newFormData = { ...formData };
    newFormData[field] = newFormData[field].filter((_, i) => i !== index);
    handleFormChange(newFormData);
  };

  const updateRule = (field: 'extractRules' | 'priorityRules' | 'notifyRules', index: number, value: string) => {
    const newFormData = { ...formData };
    newFormData[field] = [...newFormData[field]];
    newFormData[field][index] = value;
    handleFormChange(newFormData);
  };

  return (
    <div className="max-w-4xl mx-auto">
      <div className="flex items-center mb-6">
        <button
          onClick={() => navigate('/settings/skills')}
          className="p-2 -ml-2 text-gray-500 hover:bg-gray-100 rounded-lg mr-2"
        >
          <ArrowLeft className="w-5 h-5" />
        </button>
        <div>
          <h1 className="text-2xl font-bold text-gray-900">{skill?.name}</h1>
          {skill?.description && (
            <p className="text-gray-500">{skill.description}</p>
          )}
        </div>
      </div>

      {saveSuccess && (
        <div className="mb-4 p-3 bg-green-50 border border-green-200 rounded-lg flex items-center text-green-700">
          <CheckCircle className="w-5 h-5 mr-2" />
          {t('common.saveSuccess')}
        </div>
      )}

      <div className="bg-white border border-gray-200 rounded-lg">
        <div className="border-b border-gray-200 p-4 flex justify-between items-center">
          <div className="text-sm text-gray-500">
            {isFormMode ? '表单模式' : 'SKILL.md'}
          </div>
          <div className="flex items-center space-x-3">
            <button
              onClick={handleReset}
              disabled={!hasChanges}
              className="flex items-center px-3 py-1.5 text-gray-600 hover:text-gray-800 disabled:opacity-50 border border-gray-300 rounded-lg hover:bg-gray-50"
            >
              <RotateCcw className="w-4 h-4 mr-1" />
              {t('common.reset')}
            </button>
            <button
              onClick={handleToggleMode}
              className="flex items-center px-3 py-1.5 text-gray-600 hover:text-gray-800 border border-gray-300 rounded-lg hover:bg-gray-50"
            >
              {isFormMode ? (
                <>
                  <Code className="w-4 h-4 mr-1" />
                  高级模式
                </>
              ) : (
                <>
                  <Type className="w-4 h-4 mr-1" />
                  表单模式
                </>
              )}
            </button>
            <button
              onClick={handleSave}
              disabled={!hasChanges || saveMutation.isPending}
              className="flex items-center px-4 py-1.5 bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:opacity-50"
            >
              <Save className="w-4 h-4 mr-1" />
              {saveMutation.isPending ? t('common.saving') : t('common.save')}
            </button>
          </div>
        </div>

        {isFormMode ? (
          <div className="p-6 space-y-6">
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-2">身份描述 (identity)</label>
              <textarea
                value={formData.identity}
                onChange={(e) => handleFormChange({ ...formData, identity: e.target.value })}
                className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                rows={3}
                placeholder="描述这个技能的身份和用途..."
              />
            </div>

            <div>
              <div className="flex justify-between items-center mb-2">
                <label className="block text-sm font-medium text-gray-700">提取规则 (extract-rules)</label>
                <button
                  onClick={() => addRule('extractRules')}
                  className="flex items-center px-3 py-1 text-sm text-blue-600 hover:text-blue-800"
                >
                  <Plus className="w-4 h-4 mr-1" />
                  添加规则
                </button>
              </div>
              <div className="space-y-2">
                {formData.extractRules.map((rule, index) => (
                  <div key={index} className="flex items-center space-x-2">
                    <input
                      type="text"
                      value={rule}
                      onChange={(e) => updateRule('extractRules', index, e.target.value)}
                      className="flex-1 px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                      placeholder="输入规则..."
                    />
                    <button
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
                  onClick={() => addRule('priorityRules')}
                  className="flex items-center px-3 py-1 text-sm text-blue-600 hover:text-blue-800"
                >
                  <Plus className="w-4 h-4 mr-1" />
                  添加规则
                </button>
              </div>
              <div className="space-y-2">
                {formData.priorityRules.map((rule, index) => (
                  <div key={index} className="flex items-center space-x-2">
                    <input
                      type="text"
                      value={rule}
                      onChange={(e) => updateRule('priorityRules', index, e.target.value)}
                      className="flex-1 px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                      placeholder="输入规则..."
                    />
                    <button
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
                  onClick={() => addRule('notifyRules')}
                  className="flex items-center px-3 py-1 text-sm text-blue-600 hover:text-blue-800"
                >
                  <Plus className="w-4 h-4 mr-1" />
                  添加规则
                </button>
              </div>
              <div className="space-y-2">
                {formData.notifyRules.map((rule, index) => (
                  <div key={index} className="flex items-center space-x-2">
                    <input
                      type="text"
                      value={rule}
                      onChange={(e) => updateRule('notifyRules', index, e.target.value)}
                      className="flex-1 px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                      placeholder="输入规则..."
                    />
                    <button
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
                value={formData.customPrompt}
                onChange={(e) => handleFormChange({ ...formData, customPrompt: e.target.value })}
                className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                rows={6}
                placeholder="输入自定义提示词..."
              />
            </div>
          </div>
        ) : (
          <textarea
            value={content}
            onChange={(e) => handleContentChange(e.target.value)}
            className="w-full h-[600px] p-4 font-mono text-sm resize-none border-none focus:ring-0"
            spellCheck={false}
          />
        )}
      </div>

      <div className="mt-6 bg-gray-50 border border-gray-200 rounded-lg p-4">
        <h3 className="text-sm font-medium text-gray-700 mb-2">Skill {t('skill.description')}</h3>
        <ul className="text-sm text-gray-600 space-y-1">
          <li>• identity: Skill {t('skill.identityDesc')}</li>
          <li>• extract-rules: {t('skill.extractRulesDesc')}</li>
          <li>• priority-rules: {t('skill.priorityRulesDesc')}</li>
          <li>• notify-rules: {t('skill.notifyRulesDesc')}</li>
          <li>• custom-prompt: {t('skill.customPromptDesc')}</li>
        </ul>
      </div>
    </div>
  );
};

export default SkillEditor;
