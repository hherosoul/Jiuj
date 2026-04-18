import React, { useState } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { useTranslation } from 'react-i18next';
import { useNavigate } from 'react-router-dom';
import { Plus, Edit, Trash2, Zap, Star, CheckCircle } from 'lucide-react';
import ConfirmDialog from '../components/ConfirmDialog';
import {
  getAllSkills,
  getActiveSkill,
  setActiveSkill,
  createSkill,
  deleteSkill,
  Skill,
} from '../tauri-api';

const Skills: React.FC = () => {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const [showCreateForm, setShowCreateForm] = useState(false);
  const [newSkillName, setNewSkillName] = useState('');
  const [newSkillDesc, setNewSkillDesc] = useState('');
  const [addSuccess, setAddSuccess] = useState(false);
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
    mutationFn: () => createSkill(newSkillName, newSkillDesc, `# Skill: ${newSkillName}\n\n## identity\n\n## extract-rules\n\n## priority-rules\n\n## notify-rules\n\n## custom-prompt\n`),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['skills'] });
      setShowCreateForm(false);
      setNewSkillName('');
      setNewSkillDesc('');
      setAddSuccess(true);
      setTimeout(() => setAddSuccess(false), 3000);
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
    if (newSkillName) {
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

  return (
    <div className="max-w-2xl mx-auto">
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

      {addSuccess && (
        <div className="mb-4 p-3 bg-green-50 border border-green-200 rounded-lg flex items-center text-green-700">
          <CheckCircle className="w-5 h-5 mr-2" />
          {t('common.addSuccess')}
        </div>
      )}

      {showCreateForm && (
        <div className="bg-white border border-gray-200 rounded-lg p-6 mb-6">
          <form onSubmit={handleCreate}>
            <div className="space-y-4">
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">{t('skill.name')}</label>
                <input
                  type="text"
                  required
                  value={newSkillName}
                  onChange={(e) => setNewSkillName(e.target.value)}
                  className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                />
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">{t('skill.description')}</label>
                <input
                  type="text"
                  value={newSkillDesc}
                  onChange={(e) => setNewSkillDesc(e.target.value)}
                  className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                />
              </div>
            </div>
            <div className="flex justify-end space-x-3 mt-6">
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
