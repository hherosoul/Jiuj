import React, { useState, useEffect } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { useTranslation } from 'react-i18next';
import { listen } from '@tauri-apps/api/event';
import { Check, X, Mail, Ban, ChevronDown, ChevronRight } from 'lucide-react';
import ConfirmDialog from '../components/ConfirmDialog';
import {
  getPendingItems,
  completeItem,
  ignoreItem,
  getAllAccounts,
  getActiveAIProfile,
  addSkipEntry,
  Item,
} from '../tauri-api';

const Board: React.FC = () => {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const [ignoreConfirm, setIgnoreConfirm] = useState<{ isOpen: boolean; itemId: string | null }>({
    isOpen: false,
    itemId: null,
  });

  const [completeConfirm, setCompleteConfirm] = useState<{ isOpen: boolean; itemId: string | null }>({
    isOpen: false,
    itemId: null,
  });

  const [skipSenderConfirm, setSkipSenderConfirm] = useState<{ isOpen: boolean; sender: string | null }>({
    isOpen: false,
    sender: null,
  });

  const [toast, setToast] = useState<{ message: string; type: 'success' | 'error' } | null>(null);

  const [deadlineCollapsed, setDeadlineCollapsed] = useState(false);
  const [timeCollapsed, setTimeCollapsed] = useState(false);

  useEffect(() => {
    const unlisten = listen<number>('fetch-complete', (event) => {
      if (event.payload > 0) {
        queryClient.invalidateQueries({ queryKey: ['pendingItems'] });
      }
    });
    return () => {
      unlisten.then(fn => fn());
    };
  }, [queryClient]);

  const { data: pendingItems = [] } = useQuery({
    queryKey: ['pendingItems'],
    queryFn: getPendingItems,
    refetchInterval: 10000,
    select: (items) => {
      return items
        .filter(item => item.status !== 'overdue')
        .sort((a, b) => {
          const dateA = a.deadline || a.time || a.createdAt;
          const dateB = b.deadline || b.time || b.createdAt;
          if (!dateA) return 1;
          if (!dateB) return -1;
          return new Date(dateA).getTime() - new Date(dateB).getTime();
        });
    },
  });

  const { data: accounts = [] } = useQuery({
    queryKey: ['accounts'],
    queryFn: getAllAccounts,
  });

  const { data: activeProfile } = useQuery({
    queryKey: ['activeAIProfile'],
    queryFn: getActiveAIProfile,
  });

  const completeMutation = useMutation({
    mutationFn: completeItem,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['pendingItems'] });
      setCompleteConfirm({ isOpen: false, itemId: null });
    },
  });

  const ignoreMutation = useMutation({
    mutationFn: ignoreItem,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['pendingItems'] });
      setIgnoreConfirm({ isOpen: false, itemId: null });
    },
  });

  const skipSenderMutation = useMutation({
    mutationFn: (from: string) => addSkipEntry({ type: 'sender', value: from }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['pendingItems'] });
      setSkipSenderConfirm({ isOpen: false, sender: null });
      setToast({ message: t('skipList.added'), type: 'success' });
      setTimeout(() => setToast(null), 3000);
    },
    onError: (error: any) => {
      const msg = error?.toString() || '';
      if (msg.includes('UNIQUE') || msg.includes('already exists') || msg.includes('duplicate')) {
        setToast({ message: t('skipList.alreadyExists'), type: 'error' });
      } else {
        setToast({ message: t('skipList.addFailed'), type: 'error' });
      }
      setSkipSenderConfirm({ isOpen: false, sender: null });
      setTimeout(() => setToast(null), 3000);
    },
  });

  const isEmpty = pendingItems.length === 0;
  const hasNoEmails = accounts.length === 0;
  const hasNoAI = !activeProfile;

  const deadlineItems = pendingItems.filter(item => !!item.deadline);
  const timeItems = pendingItems.filter(item => !item.deadline && !!item.time);

  const getPriorityColor = (priority: string) => {
    switch (priority) {
      case 'high':
        return 'border-red-500 bg-red-50';
      case 'medium':
        return 'border-yellow-500 bg-yellow-50';
      case 'low':
        return 'border-green-500 bg-green-50';
      default:
        return 'border-gray-300 bg-white';
    }
  };

  const handleIgnoreClick = (item: Item) => {
    setIgnoreConfirm({ isOpen: true, itemId: item.id });
  };

  const handleConfirmIgnore = () => {
    if (ignoreConfirm.itemId) {
      ignoreMutation.mutate(ignoreConfirm.itemId);
    }
  };

  const handleCompleteClick = (item: Item) => {
    setCompleteConfirm({ isOpen: true, itemId: item.id });
  };

  const handleConfirmComplete = () => {
    if (completeConfirm.itemId) {
      completeMutation.mutate(completeConfirm.itemId);
    }
  };

  const handleSkipSenderClick = (sender: string) => {
    setSkipSenderConfirm({ isOpen: true, sender });
  };

  const handleConfirmSkipSender = () => {
    if (skipSenderConfirm.sender) {
      skipSenderMutation.mutate(skipSenderConfirm.sender);
    }
  };

  const formatDate = (dateStr?: string) => {
    if (!dateStr) return null;
    try {
      const date = new Date(dateStr);
      const month = date.getMonth() + 1;
      const day = date.getDate();
      const hours = date.getHours().toString().padStart(2, '0');
      const minutes = date.getMinutes().toString().padStart(2, '0');
      return `${month}/${day} ${hours}:${minutes}`;
    } catch {
      return null;
    }
  };

  const isDateOnly = (dateStr: string) => {
    try {
      const d = new Date(dateStr);
      return d.getHours() === 0 && d.getMinutes() === 0 && d.getSeconds() === 0;
    } catch {
      return false;
    }
  };

  const formatDateOnly = (dateStr: string) => {
    try {
      const d = new Date(dateStr);
      return `${d.getMonth() + 1}/${d.getDate()}`;
    } catch {
      return dateStr;
    }
  };

  const renderItem = (item: Item) => (
    <div
      key={item.id}
      className={`border-l-4 rounded-lg p-3 shadow-sm hover:shadow-md transition-shadow ${getPriorityColor(item.priority)}`}
    >
      <div className="flex justify-between items-center">
        <div className="flex-1 min-w-0">
          <p className="text-gray-900 font-medium truncate">{item.content}</p>
          {(item.deadline || item.time) && (
            <div className="flex items-center gap-2 mt-1">
              {item.deadline && (
                <span className="shrink-0 text-xs text-red-600 bg-red-100 px-2 py-0.5 rounded">
                  截止 {isDateOnly(item.deadline) ? formatDateOnly(item.deadline) : formatDate(item.deadline)}
                </span>
              )}
              {item.time && (
                <span className="shrink-0 text-xs text-blue-600 bg-blue-100 px-2 py-0.5 rounded">
                  {isDateOnly(item.time) ? formatDateOnly(item.time) : formatDate(item.time)}
                </span>
              )}
            </div>
          )}
          <div className="flex items-center mt-1 text-xs text-gray-400">
            <Mail className="w-3 h-3 mr-1" />
            {item.sourceFrom}
            {item.sourceDate && (
              <span className="ml-2">{formatDate(item.sourceDate)}</span>
            )}
          </div>
        </div>
        <div className="flex items-center gap-1 ml-3 shrink-0">
          <button
            onClick={() => handleCompleteClick(item)}
            className="p-1.5 text-green-600 hover:text-green-800 hover:bg-green-100 rounded flex items-center gap-1"
            title={t('item.complete')}
          >
            <Check className="w-4 h-4" />
            <span className="text-xs">{t('item.complete')}</span>
          </button>
          <button
            onClick={() => handleIgnoreClick(item)}
            className="p-1.5 text-gray-400 hover:text-gray-600 hover:bg-gray-200 rounded flex items-center gap-1"
            title={t('item.ignore')}
          >
            <X className="w-4 h-4" />
            <span className="text-xs">{t('item.ignore')}</span>
          </button>
          <button
            onClick={() => handleSkipSenderClick(item.sourceFrom)}
            className="p-1.5 text-orange-400 hover:text-orange-600 hover:bg-orange-100 rounded flex items-center gap-1"
            title={t('item.skipSender')}
          >
            <Ban className="w-4 h-4" />
            <span className="text-xs">{t('item.skipSender')}</span>
          </button>
        </div>
      </div>
    </div>
  );

  if (isEmpty) {
    return (
      <div className="flex flex-col items-center justify-center h-full text-center">
        <Mail className="w-16 h-16 text-gray-300 mb-4" />
        <h2 className="text-xl font-semibold text-gray-900 mb-2">{t('board.empty.title')}</h2>
        <p className="text-gray-500 mb-6">{t('board.empty.description')}</p>
        {hasNoEmails && (
          <p className="text-red-500 mb-2">{t('board.empty.noEmails')}</p>
        )}
        {hasNoAI && (
          <p className="text-red-500 mb-6">{t('board.empty.noAI')}</p>
        )}
      </div>
    );
  }

  return (
    <div className="max-w-4xl mx-auto">
      <div className="mb-6">
        <h1 className="text-2xl font-bold text-gray-900">{t('board.title')}</h1>
      </div>

      <div className="space-y-4">
        {deadlineItems.length > 0 && (
          <div>
            <button
              onClick={() => setDeadlineCollapsed(!deadlineCollapsed)}
              className="flex items-center text-sm font-medium text-gray-700 hover:text-gray-900 mb-2"
            >
              {deadlineCollapsed ? <ChevronRight className="w-4 h-4 mr-1" /> : <ChevronDown className="w-4 h-4 mr-1" />}
              {t('board.deadlineItems')} ({deadlineItems.length})
            </button>
            {!deadlineCollapsed && (
              <div className="space-y-3">
                {deadlineItems.map(renderItem)}
              </div>
            )}
          </div>
        )}

        {timeItems.length > 0 && (
          <div>
            <button
              onClick={() => setTimeCollapsed(!timeCollapsed)}
              className="flex items-center text-sm font-medium text-gray-700 hover:text-gray-900 mb-2"
            >
              {timeCollapsed ? <ChevronRight className="w-4 h-4 mr-1" /> : <ChevronDown className="w-4 h-4 mr-1" />}
              {t('board.timeItems')} ({timeItems.length})
            </button>
            {!timeCollapsed && (
              <div className="space-y-3">
                {timeItems.map(renderItem)}
              </div>
            )}
          </div>
        )}
      </div>

      <ConfirmDialog
        isOpen={ignoreConfirm.isOpen}
        title={t('confirm.title')}
        message={t('confirm.ignoreItem')}
        confirmText={t('item.ignore')}
        cancelText={t('common.cancel')}
        onConfirm={handleConfirmIgnore}
        onCancel={() => setIgnoreConfirm({ isOpen: false, itemId: null })}
        isDanger
      />

      <ConfirmDialog
        isOpen={completeConfirm.isOpen}
        title={t('confirm.title')}
        message={t('confirm.completeItem')}
        confirmText={t('item.complete')}
        cancelText={t('common.cancel')}
        onConfirm={handleConfirmComplete}
        onCancel={() => setCompleteConfirm({ isOpen: false, itemId: null })}
      />

      <ConfirmDialog
        isOpen={skipSenderConfirm.isOpen}
        title={t('confirm.title')}
        message={t('confirm.addToSkipList')}
        confirmText={t('item.skipSender')}
        cancelText={t('common.cancel')}
        onConfirm={handleConfirmSkipSender}
        onCancel={() => setSkipSenderConfirm({ isOpen: false, sender: null })}
        isDanger
      />

      {toast && (
        <div
          className={`fixed bottom-6 left-1/2 -translate-x-1/2 px-4 py-2 rounded-lg shadow-lg text-sm text-white z-[100001] ${
            toast.type === 'success' ? 'bg-green-600' : 'bg-red-500'
          }`}
        >
          {toast.message}
        </div>
      )}
    </div>
  );
};

export default Board;
