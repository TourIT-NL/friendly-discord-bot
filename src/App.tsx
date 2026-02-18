import React, { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { useAuthStore } from './store/authStore';
import { motion, AnimatePresence } from 'framer-motion';

interface DiscordUser {
  id: string;
  username: string;
  avatar?: string;
  email?: string;
}

interface Guild {
  id: string;
  name: string;
  icon?: string;
}

interface Channel {
  id: string;
  name: string;
}

interface DeletionProgress {
  current_channel: number;
  total_channels: number;
  channel_id: string;
  deleted_count: number;
  status: 'fetching' | 'deleting';
}

function App() {
  const { 
    isAuthenticated, 
    needsCredentials, 
    user, 
    guilds, 
    isLoading, 
    error, 
    setAuthenticated, 
    setUnauthenticated, 
    setLoading, 
    setError, 
    setGuilds, 
    setNeedsCredentials 
  } = useAuthStore();
  
  const [selectedGuild, setSelectedGuild] = useState<Guild | null>(null);
  const [channels, setChannels] = useState<Channel[] | null>(null);
  const [selectedChannels, setSelectedChannels] = useState<Set<string>>(new Set());
  const [isDeleting, setIsDeleting] = useState(false);
  const [deletionProgress, setDeletionProgress] = useState<DeletionProgress | null>(null);
  const [showConfirmModal, setShowConfirmModal] = useState(false);
  const [confirmText, setConfirmText] = useState('');
  const [timeRange, setTimeRange] = useState<'24h' | '7d' | 'all'>('all');

  // Credentials Setup State
  const [clientId, setClientId] = useState('');
  const [clientSecret, setClientSecret] = useState('');

  const fetchGuilds = async () => {
    try {
      setLoading(true);
      const fetchedGuilds: Guild[] = await invoke('fetch_guilds');
      setGuilds(fetchedGuilds);
    } catch (err: any) {
      console.error("Error fetching guilds:", err);
      setError(err.message || "Failed to fetch guilds.");
    } finally {
      setLoading(false);
    }
  };

  const handleSelectGuild = async (guild: Guild) => {
    setSelectedGuild(guild);
    setChannels(null);
    setSelectedChannels(new Set());
    try {
      setLoading(true);
      const fetchedChannels: Channel[] = await invoke('fetch_channels', { guildId: guild.id });
      setChannels(fetchedChannels);
    } catch (err: any) {
      console.error("Error fetching channels:", err);
      setError(err.message || "Failed to fetch channels.");
    } finally {
      setLoading(false);
    }
  };

  const toggleChannel = (channelId: string) => {
    const next = new Set(selectedChannels);
    if (next.has(channelId)) {
      next.delete(channelId);
    } else {
      next.add(channelId);
    }
    setSelectedChannels(next);
  };

  const startDeletion = async () => {
    if (confirmText !== 'DELETE') return;
    
    setShowConfirmModal(false);
    setIsDeleting(true);
    setDeletionProgress(null);
    
    const now = Date.now();
    let startTime: number | undefined;
    if (timeRange === '24h') startTime = now - 24 * 60 * 60 * 1000;
    else if (timeRange === '7d') startTime = now - 7 * 24 * 60 * 60 * 1000;

    try {
      await invoke('bulk_delete_messages', {
        channelIds: Array.from(selectedChannels),
        startTime,
        endTime: undefined
      });
    } catch (err: any) {
      console.error("Error during deletion:", err);
      setError(err.message || "An error occurred during deletion.");
      setIsDeleting(false);
    }
  };

  useEffect(() => {
    const unlistenStarted = listen('auth_started', () => {
      setLoading(true);
      setError(null);
    });

    const unlistenSuccess = listen('auth_success', (event) => {
      const userProfile = event.payload as DiscordUser;
      setAuthenticated(userProfile);
      fetchGuilds();
    });

    const unlistenProgress = listen('deletion_progress', (event) => {
      setDeletionProgress(event.payload as DeletionProgress);
    });

    const unlistenComplete = listen('deletion_complete', () => {
      setIsDeleting(false);
      setDeletionProgress(null);
    });

    return () => {
      unlistenStarted.then(f => f());
      unlistenSuccess.then(f => f());
      unlistenProgress.then(f => f());
      unlistenComplete.then(f => f());
    };
  }, [setAuthenticated, setLoading, setError, setGuilds]);

  const handleLogin = async () => {
    setLoading(true);
    setError(null);
    try {
      const userProfile: DiscordUser = await invoke('start_oauth_flow');
      setAuthenticated(userProfile);
      await fetchGuilds();
    } catch (err: any) {
      console.error("Error during OAuth flow:", err);
      if (err.error_code === 'credentials_missing') {
        setNeedsCredentials(true);
      } else {
        setUnauthenticated();
        setError(err.message || "An unknown error occurred during login.");
      }
    } finally {
      setLoading(false);
    }
  };

  const handleSaveCredentials = async (e: React.FormEvent) => {
    e.preventDefault();
    setLoading(true);
    setError(null);
    try {
      await invoke('save_discord_credentials', { clientId, clientSecret });
      setNeedsCredentials(false);
      // Automatically trigger login after saving credentials
      handleLogin();
    } catch (err: any) {
      console.error("Failed to save credentials:", err);
      setError(err.message || "Failed to save credentials.");
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="min-h-screen bg-gray-900 text-white flex flex-col items-center py-12 px-4 selection:bg-blue-500/30">
      <motion.h1 
        initial={{ y: -20, opacity: 0 }}
        animate={{ y: 0, opacity: 1 }}
        className="text-4xl font-black mb-8 tracking-tight bg-gradient-to-r from-blue-400 to-purple-500 bg-clip-text text-transparent"
      >
        Discord Privacy Utility
      </motion.h1>

      <AnimatePresence>
        {isLoading && !isDeleting && (
          <motion.div 
            initial={{ opacity: 0, x: 20 }}
            animate={{ opacity: 1, x: 0 }}
            exit={{ opacity: 0, x: 20 }}
            className="fixed top-4 right-4 bg-blue-600 px-4 py-2 rounded-lg shadow-lg z-50 flex items-center gap-2"
          >
            <div className="w-4 h-4 border-2 border-white/30 border-t-white rounded-full animate-spin" />
            <p className="font-semibold text-sm">Loading...</p>
          </motion.div>
        )}
      </AnimatePresence>

      {error && (
        <motion.div 
          initial={{ opacity: 0, scale: 0.95 }}
          animate={{ opacity: 1, scale: 1 }}
          className="bg-red-900/20 border border-red-500/50 text-red-200 p-4 mb-8 w-full max-w-2xl rounded-xl backdrop-blur-sm"
        >
          <p className="font-bold flex items-center gap-2">
            <span className="text-xl">⚠️</span> Error
          </p>
          <p className="mt-1 opacity-90">{error}</p>
        </motion.div>
      )}

      {needsCredentials ? (
        <motion.div 
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          className="bg-gray-800/50 border border-gray-700 p-8 rounded-3xl shadow-2xl max-w-md w-full"
        >
          <h2 className="text-2xl font-bold mb-6 text-center">Initial Setup</h2>
          <p className="text-gray-400 mb-8 text-sm text-center">
            You need to provide your Discord application credentials. These will be stored securely in your system's keyring.
          </p>
          <form onSubmit={handleSaveCredentials} className="space-y-4">
            <div className="space-y-1">
              <label className="text-xs font-bold text-gray-500 uppercase tracking-widest ml-1">Client ID</label>
              <input
                type="text"
                required
                value={clientId}
                onChange={(e) => setClientId(e.target.value)}
                placeholder="123456789012345678"
                className="w-full bg-gray-900 border border-gray-700 p-4 rounded-xl focus:outline-none focus:border-blue-500/50 transition-colors"
              />
            </div>
            <div className="space-y-1">
              <label className="text-xs font-bold text-gray-500 uppercase tracking-widest ml-1">Client Secret</label>
              <input
                type="password"
                required
                value={clientSecret}
                onChange={(e) => setClientSecret(e.target.value)}
                placeholder="••••••••••••••••"
                className="w-full bg-gray-900 border border-gray-700 p-4 rounded-xl focus:outline-none focus:border-blue-500/50 transition-colors"
              />
            </div>
            <button
              type="submit"
              className="w-full bg-blue-600 hover:bg-blue-700 text-white font-bold py-4 rounded-xl shadow-xl shadow-blue-500/10 transition-all active:scale-95"
            >
              Save & Continue
            </button>
            <p className="text-[10px] text-gray-600 text-center mt-4">
              Find these in the <a href="https://discord.com/developers/applications" target="_blank" className="text-blue-500 underline">Discord Developer Portal</a> under 'OAuth2' -&gt; 'General'.
            </p>
          </form>
        </motion.div>
      ) : !isAuthenticated && !isLoading ? (
        <motion.div 
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          className="flex flex-col items-center"
        >
          <p className="text-gray-400 mb-8 max-w-md text-center text-lg">
            Reclaim your digital footprint. Securely manage your messages and server memberships with ease.
          </p>
          <button
            onClick={handleLogin}
            className="bg-blue-600 hover:bg-blue-700 text-white font-bold py-4 px-10 rounded-2xl shadow-xl shadow-blue-500/20 transition-all hover:scale-105 active:scale-95 flex items-center gap-3"
          >
            Login with Discord
          </button>
        </motion.div>
      ) : isAuthenticated ? (
        <div className="w-full max-w-5xl space-y-8">
          {/* Header & Profile */}
          <motion.div 
            initial={{ opacity: 0, y: 10 }}
            animate={{ opacity: 1, y: 0 }}
            className="flex items-center justify-between bg-gray-800/50 border border-gray-700 p-6 rounded-2xl backdrop-blur-md shadow-xl"
          >
            <div className="flex items-center gap-4">
              {user?.avatar ? (
                <img
                  src={`https://cdn.discordapp.com/avatars/${user.id}/${user.avatar}.png`}
                  alt="User Avatar"
                  className="w-14 h-14 rounded-full border-2 border-blue-500/50"
                />
              ) : (
                <div className="w-14 h-14 rounded-full bg-blue-600 flex items-center justify-center font-bold text-xl">
                  {user?.username.charAt(0)}
                </div>
              )}
              <div>
                <h2 className="text-xl font-bold">{user?.username}</h2>
                <p className="text-gray-500 text-xs font-mono uppercase tracking-widest">Authenticated</p>
              </div>
            </div>
            <button
              onClick={setUnauthenticated}
              className="text-gray-400 hover:text-red-400 transition-colors text-sm font-medium px-4 py-2 hover:bg-red-400/10 rounded-lg"
            >
              Logout
            </button>
          </motion.div>

          <div className="grid grid-cols-1 lg:grid-cols-3 gap-8">
            {/* Guild List */}
            <div className="lg:col-span-1 space-y-4">
              <h3 className="text-sm font-bold text-gray-500 uppercase tracking-widest ml-1">Select Server</h3>
              <div className="bg-gray-800/30 border border-gray-700 rounded-2xl overflow-hidden max-h-[600px] overflow-y-auto custom-scrollbar">
                {guilds?.map((guild) => (
                  <button
                    key={guild.id}
                    onClick={() => handleSelectGuild(guild)}
                    className={`w-full flex items-center gap-3 p-4 transition-all border-b border-gray-700 last:border-0 ${selectedGuild?.id === guild.id ? 'bg-blue-600/20 text-blue-400' : 'hover:bg-gray-700/50 text-gray-300'}`}
                  >
                    {guild.icon ? (
                      <img
                        src={`https://cdn.discordapp.com/icons/${guild.id}/${guild.icon}.png`}
                        alt={guild.name}
                        className="w-10 h-10 rounded-lg shadow-lg"
                      />
                    ) : (
                      <div className="w-10 h-10 rounded-lg bg-gray-700 flex items-center justify-center font-bold text-gray-500">
                        {guild.name.charAt(0)}
                      </div>
                    )}
                    <span className="font-semibold text-left truncate">{guild.name}</span>
                  </button>
                ))}
              </div>
            </div>

            {/* Channels & Actions */}
            <div className="lg:col-span-2 space-y-6">
              {selectedGuild ? (
                <motion.div 
                  initial={{ opacity: 0, x: 20 }}
                  animate={{ opacity: 1, x: 0 }}
                  className="space-y-6"
                >
                  <div className="bg-gray-800/50 border border-gray-700 p-8 rounded-2xl space-y-6">
                    <div className="flex items-center justify-between">
                      <h3 className="text-2xl font-bold text-white flex items-center gap-3">
                        {selectedGuild.name}
                        {channels && (
                          <span className="text-sm font-normal text-gray-500 bg-gray-700 px-3 py-1 rounded-full">
                            {channels.length} text channels
                          </span>
                        )}
                      </h3>
                    </div>

                    {/* Time Range */}
                    <div className="space-y-3">
                      <label className="text-xs font-bold text-gray-500 uppercase tracking-widest ml-1">Deletion Timeframe</label>
                      <div className="flex gap-2 p-1 bg-gray-900/50 rounded-xl border border-gray-700">
                        {(['24h', '7d', 'all'] as const).map((r) => (
                          <button
                            key={r}
                            onClick={() => setTimeRange(r)}
                            className={`flex-1 py-2 px-4 rounded-lg font-bold text-sm transition-all ${timeRange === r ? 'bg-blue-600 text-white shadow-lg' : 'text-gray-500 hover:text-gray-300'}`}
                          >
                            {r === '24h' ? 'Last 24 Hours' : r === '7d' ? 'Last 7 Days' : 'All Time'}
                          </button>
                        ))}
                      </div>
                    </div>

                    {/* Channel List */}
                    <div className="space-y-3">
                      <div className="flex items-center justify-between ml-1">
                        <label className="text-xs font-bold text-gray-500 uppercase tracking-widest">Select Channels</label>
                        <button 
                          onClick={() => setSelectedChannels(new Set(channels?.map(c => c.id)))}
                          className="text-xs text-blue-400 hover:text-blue-300 transition-colors"
                        >
                          Select All
                        </button>
                      </div>
                      <div className="grid grid-cols-1 md:grid-cols-2 gap-3 max-h-[300px] overflow-y-auto pr-2 custom-scrollbar">
                        {channels?.map((channel) => (
                          <button
                            key={channel.id}
                            onClick={() => toggleChannel(channel.id)}
                            className={`flex items-center gap-3 p-3 rounded-xl border transition-all text-left ${selectedChannels.has(channel.id) ? 'bg-blue-600/10 border-blue-500/50 text-blue-200' : 'bg-gray-900/30 border-gray-700 text-gray-400 hover:border-gray-600'}`}
                          >
                            <div className={`w-5 h-5 rounded flex items-center justify-center border transition-colors ${selectedChannels.has(channel.id) ? 'bg-blue-500 border-blue-500 text-white' : 'border-gray-600'}`}>
                              {selectedChannels.has(channel.id) && <span className="text-[10px] font-black">✓</span>}
                            </div>
                            <span className="truncate font-medium">#{channel.name}</span>
                          </button>
                        ))}
                      </div>
                    </div>

                    <div className="pt-4">
                      <button
                        disabled={selectedChannels.size === 0 || isDeleting}
                        onClick={() => setShowConfirmModal(true)}
                        className={`w-full py-4 rounded-xl font-bold text-lg shadow-xl transition-all ${selectedChannels.size > 0 && !isDeleting ? 'bg-red-600 hover:bg-red-700 text-white shadow-red-500/10 hover:scale-[1.02]' : 'bg-gray-700 text-gray-500 cursor-not-allowed opacity-50'}`}
                      >
                        {isDeleting ? 'Processing...' : `Delete Messages from ${selectedChannels.size} Channels`}
                      </button>
                    </div>
                  </div>
                </motion.div>
              ) : (
                <div className="h-full flex flex-col items-center justify-center bg-gray-800/20 border border-gray-700 border-dashed rounded-3xl p-12 text-center">
                  <div className="text-4xl mb-4 opacity-20">🛡️</div>
                  <h3 className="text-xl font-bold text-gray-500 mb-2">No Server Selected</h3>
                  <p className="text-gray-600 max-w-xs">Please select a server from the sidebar to begin the cleanup process.</p>
                </div>
              )}
            </div>
          </div>
        </div>
      ) : null}

      {/* Confirmation Modal */}
      <AnimatePresence>
        {showConfirmModal && (
          <motion.div 
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            className="fixed inset-0 bg-black/80 backdrop-blur-sm z-[100] flex items-center justify-center p-6"
          >
            <motion.div 
              initial={{ scale: 0.9, y: 20 }}
              animate={{ scale: 1, y: 0 }}
              className="bg-gray-800 border border-red-500/30 rounded-3xl p-10 w-full max-w-md shadow-2xl space-y-6"
            >
              <div className="text-center space-y-2">
                <div className="w-16 h-16 bg-red-600/20 text-red-500 rounded-full flex items-center justify-center mx-auto mb-4 text-3xl">⚠️</div>
                <h2 className="text-2xl font-black">Hold On!</h2>
                <p className="text-gray-400">
                  This will permanently delete all messages from <span className="text-white font-bold">{selectedChannels.size} channels</span> in <span className="text-white font-bold">{selectedGuild?.name}</span>.
                </p>
                <p className="text-sm text-red-400 bg-red-400/10 p-2 rounded-lg mt-4 font-semibold uppercase tracking-widest">This action is irreversible.</p>
              </div>

              <div className="space-y-3">
                <label className="text-xs font-bold text-gray-500 uppercase tracking-widest ml-1">Type <span className="text-red-400">DELETE</span> to confirm</label>
                <input
                  type="text"
                  value={confirmText}
                  onChange={(e) => setConfirmText(e.target.value)}
                  placeholder="DELETE"
                  className="w-full bg-gray-900 border border-gray-700 p-4 rounded-xl text-center font-black tracking-widest text-red-500 focus:outline-none focus:border-red-500/50 transition-colors"
                />
              </div>

              <div className="flex gap-4">
                <button
                  onClick={() => setShowConfirmModal(false)}
                  className="flex-1 py-4 text-gray-400 font-bold hover:text-white transition-colors"
                >
                  Cancel
                </button>
                <button
                  disabled={confirmText !== 'DELETE'}
                  onClick={startDeletion}
                  className={`flex-1 py-4 rounded-xl font-bold transition-all ${confirmText === 'DELETE' ? 'bg-red-600 hover:bg-red-700 text-white' : 'bg-gray-700 text-gray-500 cursor-not-allowed'}`}
                >
                  Proceed
                </button>
              </div>
            </motion.div>
          </motion.div>
        )}
      </AnimatePresence>

      {/* Progress Overlay */}
      <AnimatePresence>
        {isDeleting && (
          <motion.div 
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            className="fixed inset-0 bg-gray-900/95 backdrop-blur-md z-[110] flex flex-col items-center justify-center p-8 text-center"
          >
            <div className="w-full max-w-lg space-y-12">
              <div className="space-y-4">
                <h2 className="text-4xl font-black tracking-tighter">Cleaning Up...</h2>
                <p className="text-blue-400 font-mono text-sm uppercase tracking-widest font-bold">Progress Dashboard</p>
              </div>

              <div className="space-y-8">
                {deletionProgress ? (
                  <>
                    <div className="space-y-4">
                      <div className="flex justify-between text-sm font-bold text-gray-500 px-2 uppercase tracking-widest">
                        <span>Overall Progress</span>
                        <span>Channel {deletionProgress.current_channel} of {deletionProgress.total_channels}</span>
                      </div>
                      <div className="w-full h-4 bg-gray-800 rounded-full overflow-hidden border border-gray-700 p-1">
                        <motion.div 
                          initial={{ width: 0 }}
                          animate={{ width: `${(deletionProgress.current_channel / deletionProgress.total_channels) * 100}%` }}
                          className="h-full bg-gradient-to-r from-blue-500 to-purple-500 rounded-full shadow-[0_0_15px_rgba(59,130,246,0.5)]"
                        />
                      </div>
                    </div>

                    <div className="grid grid-cols-2 gap-8">
                      <div className="bg-gray-800/50 p-6 rounded-2xl border border-gray-700">
                        <p className="text-xs font-bold text-gray-500 uppercase tracking-widest mb-1">Status</p>
                        <p className="text-xl font-bold text-blue-400 capitalize">{deletionProgress.status}...</p>
                      </div>
                      <div className="bg-gray-800/50 p-6 rounded-2xl border border-gray-700">
                        <p className="text-xs font-bold text-gray-500 uppercase tracking-widest mb-1">Messages Deleted</p>
                        <p className="text-xl font-bold text-purple-400">{deletionProgress.deleted_count}</p>
                      </div>
                    </div>

                    <p className="text-gray-500 text-sm italic">
                      Currently processing <span className="text-white font-bold">#{channels?.find(c => c.id === deletionProgress.channel_id)?.name || 'unknown'}</span>
                    </p>
                  </>
                ) : (
                  <div className="space-y-6">
                    <div className="w-16 h-16 border-4 border-blue-500/20 border-t-blue-500 rounded-full animate-spin mx-auto" />
                    <p className="text-gray-400 animate-pulse">Initializing connection to Discord API...</p>
                  </div>
                )}
              </div>
            </div>
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
}

export default App;
