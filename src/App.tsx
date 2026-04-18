import { useState, useEffect } from 'react';
import { Routes, Route, useNavigate, useLocation } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { listen } from '@tauri-apps/api/event';
import { requestPermission, isPermissionGranted } from '@tauri-apps/plugin-notification';
import {
  LayoutDashboard,
  Mail,
  Settings,
  Brain,
  Zap,
  List,
  ChevronLeft,
  RefreshCw,
  CheckCircle,
  AlertCircle,
  Bell,
  X,
} from 'lucide-react';
import Board from './pages/Board';
import Accounts from './pages/Accounts';
import AISettings from './pages/AISettings';
import Skills from './pages/Skills';
import SkipList from './pages/SkipList';
import General from './pages/General';
import SkillEditor from './pages/SkillEditor';
import { triggerFetchNow } from './tauri-api';

interface ReminderAlert {
  title: string;
  body: string;
  timestamp: number;
}

function App() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const location = useLocation();
  const [sidebarOpen, setSidebarOpen] = useState(true);
  const [fetching, setFetching] = useState(false);
  const [fetchSuccess, setFetchSuccess] = useState(false);
  const [fetchError, setFetchError] = useState<string | null>(null);
  const [fetchStatus, setFetchStatus] = useState<string | null>(null);
  const [reminders, setReminders] = useState<ReminderAlert[]>([]);

  useEffect(() => {
    (async () => {
      try {
        let granted = await isPermissionGranted();
        if (!granted) {
          const permission = await requestPermission();
          granted = permission === 'granted';
          console.log('[Notification] Permission result:', permission);
        } else {
          console.log('[Notification] Permission already granted');
        }
      } catch (e) {
        console.error('[Notification] Failed to request permission:', e);
      }
    })();
  }, []);

  useEffect(() => {
    const unlistenReminder = listen<{ title: string; body: string }>('reminder-triggered', (event) => {
      const { title, body } = event.payload;
      setReminders(prev => [...prev, { title, body, timestamp: Date.now() }]);
    });

    return () => {
      unlistenReminder.then(fn => fn());
    };
  }, []);

  const dismissReminder = (timestamp: number) => {
    setReminders(prev => prev.filter(r => r.timestamp !== timestamp));
  };

  useEffect(() => {
    const unlistenStatus = listen<string>('fetch-status', (event) => {
      setFetchStatus(event.payload);
    });

    const unlistenError = listen<string>('fetch-error', (event) => {
      setFetchError(event.payload);
      setFetching(false);
      setFetchStatus(null);
      setTimeout(() => setFetchError(null), 5000);
    });

    const unlistenComplete = listen<number>('fetch-complete', (event) => {
      setFetching(false);
      setFetchStatus(null);
      if (event.payload > 0) {
        setFetchSuccess(true);
        setTimeout(() => setFetchSuccess(false), 3000);
      }
    });

    return () => {
      unlistenStatus.then(fn => fn());
      unlistenError.then(fn => fn());
      unlistenComplete.then(fn => fn());
    };
  }, []);

  const handleTriggerFetch = async () => {
    if (fetching) return;
    setFetching(true);
    setFetchSuccess(false);
    setFetchError(null);
    setFetchStatus(null);
    try {
      await triggerFetchNow();
    } catch (e) {
      setFetchError(String(e));
      setFetching(false);
      setFetchStatus(null);
      setTimeout(() => setFetchError(null), 5000);
    }
  };

  const navItems = [
    { path: '/', label: 'board.title', icon: LayoutDashboard },
    { path: '/settings/accounts', label: 'settings.accounts', icon: Mail },
    { path: '/settings/ai', label: 'settings.ai', icon: Brain },
    { path: '/settings/skills', label: 'settings.skills', icon: Zap },
    { path: '/settings/skip-list', label: 'settings.skipList', icon: List },
    { path: '/settings/general', label: 'settings.general', icon: Settings },
  ];

  const currentNavItem = navItems.find((item) => item.path === location.pathname);

  return (
    <div className="flex h-screen bg-gray-50">
      {sidebarOpen && (
        <div className="w-64 bg-white border-r border-gray-200 flex flex-col">
          <div className="p-6 border-b border-gray-100">
            <h1 className="text-xl font-bold text-gray-900">{t('app.name')}</h1>
            <p className="text-sm text-gray-500 mt-1">{t('app.tagline')}</p>
          </div>

          <nav className="flex-1 p-4 space-y-1">
            {navItems.map((item) => {
              const Icon = item.icon;
              const isActive = location.pathname === item.path;
              return (
                <button
                  key={item.path}
                  onClick={() => navigate(item.path)}
                  className={`w-full flex items-center px-4 py-3 rounded-lg text-left transition-colors ${
                    isActive
                      ? 'bg-blue-50 text-blue-700'
                      : 'text-gray-600 hover:bg-gray-50'
                  }`}
                >
                  <Icon className="w-5 h-5 mr-3" />
                  <span className="font-medium">{t(item.label)}</span>
                </button>
              );
            })}
          </nav>

          <div className="p-4 border-t border-gray-100 space-y-3">
            {fetchSuccess && (
              <div className="p-2 bg-green-50 border border-green-200 rounded-lg flex items-center justify-center text-green-700 text-sm">
                <CheckCircle className="w-4 h-4 mr-1" />
                {t('common.fetchSuccess')}
              </div>
            )}
            
            {fetchError && (
              <div className="p-2 bg-red-50 border border-red-200 rounded-lg flex items-center justify-center text-red-700 text-sm">
                <AlertCircle className="w-4 h-4 mr-1" />
                {fetchError}
              </div>
            )}
            
            {fetchStatus && fetching && (
              <div className="p-2 bg-blue-50 border border-blue-200 rounded-lg flex items-center justify-center text-blue-700 text-sm">
                {fetchStatus}
              </div>
            )}

            <button
              onClick={handleTriggerFetch}
              disabled={fetching}
              className="w-full flex items-center justify-center px-4 py-2 rounded-lg text-gray-600 hover:bg-gray-50 transition-colors disabled:opacity-50"
            >
              <RefreshCw className={`w-4 h-4 mr-2 ${fetching ? 'animate-spin' : ''}`} />
              <span className="font-medium">{fetching ? t('common.fetching') : t('common.fetchNow')}</span>
            </button>
          </div>
        </div>
      )}

      <div className="flex-1 flex flex-col min-w-0">
        <header className="h-16 bg-white border-b border-gray-200 flex items-center px-6">
          {!sidebarOpen && (
            <button
              onClick={() => setSidebarOpen(true)}
              className="p-2 -ml-2 text-gray-500 hover:bg-gray-100 rounded-lg mr-4"
            >
              <ChevronLeft className="w-5 h-5 rotate-180" />
            </button>
          )}
          <h2 className="text-lg font-semibold text-gray-900">
            {currentNavItem ? t(currentNavItem.label) : ''}
          </h2>
        </header>

        <main className="flex-1 overflow-auto p-6">
          <Routes>
            <Route path="/" element={<Board />} />
            <Route path="/settings/accounts" element={<Accounts />} />
            <Route path="/settings/ai" element={<AISettings />} />
            <Route path="/settings/skills" element={<Skills />} />
            <Route path="/skills/:id" element={<SkillEditor />} />
            <Route path="/settings/skip-list" element={<SkipList />} />
            <Route path="/settings/general" element={<General />} />
          </Routes>
        </main>
      </div>

      {reminders.length > 0 && (
        <div className="fixed top-4 right-4 z-[100000] space-y-2 max-w-sm">
          {reminders.map((reminder) => (
            <div
              key={reminder.timestamp}
              className="bg-orange-50 border border-orange-200 rounded-lg shadow-lg p-4 flex items-start gap-3 animate-slide-in"
            >
              <Bell className="w-5 h-5 text-orange-500 shrink-0 mt-0.5" />
              <div className="flex-1 min-w-0">
                <p className="font-semibold text-orange-800 text-sm">{reminder.title}</p>
                <p className="text-orange-700 text-sm mt-0.5">{reminder.body}</p>
              </div>
              <button
                onClick={() => dismissReminder(reminder.timestamp)}
                className="shrink-0 p-1 text-orange-400 hover:text-orange-600 rounded"
              >
                <X className="w-4 h-4" />
              </button>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}

export default App;
