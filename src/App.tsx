import React, { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { useAuthStore } from './store/authStore';
import { motion, AnimatePresence } from 'framer-motion';
import { QRCodeSVG } from 'qrcode.react';
import { Monitor, Smartphone, Key, Globe, ShieldCheck, ShieldAlert, CheckCircle2, XCircle } from 'lucide-react';

interface DiscordUser {
  id: string;
  username: string;
  avatar?: string;
  email?: string;
}

interface DiscordStatus {
  is_running: boolean;
  rpc_available: boolean;
  browser_detected: boolean;
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

  // Enhanced Auth State
  const [authMethod, setAuthMethod] = useState<'none' | 'oauth' | 'qr' | 'token'>('none');
  const [discordStatus, setDiscordStatus] = useState<DiscordStatus | null>(null);
  const [qrUrl, setQrUrl] = useState<string | null>(null);
  const [qrScanned, setQrScanned] = useState(false);
  const [manualToken, setManualToken] = useState('');

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

  const checkDiscordStatus = async () => {
    try {
      const status: DiscordStatus = await invoke('check_discord_status');
      setDiscordStatus(status);
    } catch (err) {
      console.error("Failed to check Discord status:", err);
    }
  };

  useEffect(() => {
    checkDiscordStatus();
    const interval = setInterval(checkDiscordStatus, 5000);
    return () => clearInterval(interval);
  }, []);

  useEffect(() => {
    const unlistenStarted = listen('auth_started', () => {
      setLoading(true);
      setError(null);
    });

    const unlistenSuccess = listen('auth_success', (event) => {
      const userProfile = event.payload as DiscordUser;
      setAuthenticated(userProfile);
      setAuthMethod('none');
      setQrUrl(null);
      setQrScanned(false);
      fetchGuilds();
    });

    const unlistenQrReady = listen<string>('qr_code_ready', (event) => {
      setQrUrl(event.payload);
      setLoading(false);
    });

    const unlistenQrScanned = listen('qr_scanned', () => {
      setQrScanned(true);
    });

    const unlistenQrCancelled = listen('qr_cancelled', () => {
      setAuthMethod('none');
      setQrUrl(null);
      setQrScanned(false);
      setError("QR Login timed out or was cancelled.");
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
      unlistenQrReady.then(f => f());
      unlistenQrScanned.then(f => f());
      unlistenQrCancelled.then(f => f());
      unlistenProgress.then(f => f());
      unlistenComplete.then(f => f());
    };
  }, [setAuthenticated, setLoading, setError, setGuilds]);

  const handleLoginOAuth = async () => {
    setLoading(true);
    setError(null);
    try {
      await invoke('start_oauth_flow');
    } catch (err: any) {
      if (err.error_code === 'credentials_missing') {
        setNeedsCredentials(true);
      } else {
        setError(err.message || "An unknown error occurred during login.");
      }
      setLoading(false);
    }
  };

  const handleLoginQR = async () => {
    setAuthMethod('qr');
    setLoading(true);
    setError(null);
    try {
      await invoke('start_qr_login_flow');
    } catch (err: any) {
      setError(err.message || "Failed to initialize QR login.");
      setLoading(false);
      setAuthMethod('none');
    }
  };

  const handleLoginToken = async (e: React.FormEvent) => {
    e.preventDefault();
    setLoading(true);
    setError(null);
    try {
      await invoke('login_with_token', { token: manualToken });
    } catch (err: any) {
      setError(err.message || "Failed to login with token.");
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
      handleLoginOAuth();
    } catch (err: any) {
      setError(err.message || "Failed to save credentials.");
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

  return (
    <div className="min-h-screen bg-gray-950 text-white flex flex-col items-center py-12 px-4 selection:bg-blue-500/30 font-sans">
      <motion.div 
        initial={{ y: -20, opacity: 0 }}
        animate={{ y: 0, opacity: 1 }}
        className="text-center mb-12"
      >
        <h1 className="text-5xl font-black tracking-tighter bg-gradient-to-br from-white via-gray-300 to-gray-600 bg-clip-text text-transparent mb-2">
          Discord Privacy Utility
        </h1>
        <p className="text-gray-500 font-medium tracking-wide text-sm uppercase">Secure Digital Footprint Manager</p>
      </motion.div>

      <AnimatePresence>
        {isLoading && !isDeleting && (
          <motion.div 
            initial={{ opacity: 0, scale: 0.9 }}
            animate={{ opacity: 1, scale: 1 }}
            exit={{ opacity: 0, scale: 0.9 }}
            className="fixed top-6 right-6 bg-blue-600/90 backdrop-blur-md px-5 py-3 rounded-2xl shadow-2xl z-50 flex items-center gap-3 border border-white/10"
          >
            <div className="w-5 h-5 border-2 border-white/20 border-t-white rounded-full animate-spin" />
            <p className="font-bold text-sm">Processing...</p>
          </motion.div>
        )}
      </AnimatePresence>

      {error && (
        <motion.div 
          initial={{ opacity: 0, y: 10 }}
          animate={{ opacity: 1, y: 0 }}
          className="bg-red-950/30 border border-red-500/30 text-red-200 p-5 mb-10 w-full max-w-2xl rounded-2xl backdrop-blur-xl flex items-start gap-4"
        >
          <div className="bg-red-500/20 p-2 rounded-lg">
            <ShieldAlert className="w-5 h-5 text-red-500" />
          </div>
          <div className="flex-1">
            <p className="font-black text-sm uppercase tracking-widest mb-1">Security Alert</p>
            <p className="text-sm opacity-90 leading-relaxed">{error}</p>
          </div>
          <button onClick={() => setError(null)} className="text-red-500/50 hover:text-red-500 transition-colors">
            <XCircle className="w-5 h-5" />
          </button>
        </motion.div>
      )}

      {needsCredentials ? (
        <motion.div 
          initial={{ opacity: 0, scale: 0.95 }}
          animate={{ opacity: 1, scale: 1 }}
          className="bg-gray-900/50 border border-gray-800 p-10 rounded-[2.5rem] shadow-2xl max-w-lg w-full backdrop-blur-3xl"
        >
          <div className="text-center mb-8">
            <div className="w-16 h-16 bg-blue-600/10 rounded-3xl flex items-center justify-center mx-auto mb-4 border border-blue-500/20">
              <Key className="w-8 h-8 text-blue-500" />
            </div>
            <h2 className="text-3xl font-black tracking-tight mb-2">App Config</h2>
            <p className="text-gray-500 text-sm">Configure your Discord API credentials to enable OAuth2 features.</p>
          </div>
          
          <form onSubmit={handleSaveCredentials} className="space-y-5">
            <div className="space-y-2">
              <label className="text-[10px] font-black text-gray-600 uppercase tracking-[0.2em] ml-2">Client ID</label>
              <input
                type="text"
                required
                value={clientId}
                onChange={(e) => setClientId(e.target.value)}
                placeholder="123456789012345678"
                className="w-full bg-black/40 border border-gray-800 p-4 rounded-2xl focus:outline-none focus:border-blue-500/50 transition-all font-mono text-sm"
              />
            </div>
            <div className="space-y-2">
              <label className="text-[10px] font-black text-gray-600 uppercase tracking-[0.2em] ml-2">Client Secret</label>
              <input
                type="password"
                required
                value={clientSecret}
                onChange={(e) => setClientSecret(e.target.value)}
                placeholder="••••••••••••••••"
                className="w-full bg-black/40 border border-gray-800 p-4 rounded-2xl focus:outline-none focus:border-blue-500/50 transition-all font-mono text-sm"
              />
            </div>
            <button
              type="submit"
              className="w-full bg-white text-black font-black py-5 rounded-2xl shadow-xl hover:bg-gray-100 transition-all active:scale-95 text-sm uppercase tracking-widest"
            >
              Initialize Engine
            </button>
            <div className="flex items-center gap-2 justify-center mt-6 p-4 bg-gray-900/50 rounded-xl border border-gray-800">
              <Globe className="w-4 h-4 text-gray-600" />
              <p className="text-[10px] text-gray-500">
                Setup available in <a href="https://discord.com/developers/applications" target="_blank" className="text-blue-500 hover:text-blue-400 font-bold">Discord Dev Portal</a>
              </p>
            </div>
          </form>
        </motion.div>
      ) : !isAuthenticated && !isLoading ? (
        <div className="w-full max-w-4xl grid grid-cols-1 md:grid-cols-2 gap-8 px-4">
          <motion.div 
            initial={{ opacity: 0, x: -20 }}
            animate={{ opacity: 1, x: 0 }}
            className="space-y-6"
          >
            <div className="bg-gray-900/40 border border-gray-800 p-8 rounded-[2rem] backdrop-blur-2xl">
              <h3 className="text-sm font-black text-gray-600 uppercase tracking-[0.2em] mb-6 flex items-center gap-2">
                <Monitor className="w-4 h-4" /> Discord Environment
              </h3>
              <div className="space-y-4">
                <div className="flex items-center justify-between p-4 bg-black/20 rounded-2xl border border-white/5">
                  <span className="text-sm font-medium text-gray-400">Desktop Client</span>
                  {discordStatus?.is_running ? (
                    <span className="flex items-center gap-2 text-green-500 text-xs font-black uppercase">
                      <CheckCircle2 className="w-4 h-4" /> Active
                    </span>
                  ) : (
                    <span className="flex items-center gap-2 text-gray-600 text-xs font-black uppercase">
                       Offline
                    </span>
                  )}
                </div>
                <div className="flex items-center justify-between p-4 bg-black/20 rounded-2xl border border-white/5">
                  <span className="text-sm font-medium text-gray-400">Instant Link (RPC)</span>
                  {discordStatus?.rpc_available ? (
                    <span className="flex items-center gap-2 text-blue-500 text-xs font-black uppercase">
                      <ShieldCheck className="w-4 h-4" /> Ready
                    </span>
                  ) : (
                    <span className="text-gray-600 text-xs font-black uppercase">Unsupported</span>
                  )}
                </div>
              </div>
            </div>

            <div className="bg-gray-900/40 border border-gray-800 p-8 rounded-[2rem] backdrop-blur-2xl text-center">
              {authMethod === 'qr' ? (
                <div className="space-y-6">
                  <h3 className="text-lg font-black tracking-tight">QR Login</h3>
                  <div className="bg-white p-4 rounded-3xl inline-block shadow-2xl shadow-blue-500/20">
                    {qrUrl ? (
                      <QRCodeSVG value={qrUrl} size={180} level="H" includeMargin={true} />
                    ) : (
                      <div className="w-[180px] h-[180px] flex items-center justify-center bg-gray-100 rounded-xl">
                        <div className="w-8 h-8 border-4 border-blue-500/20 border-t-blue-500 rounded-full animate-spin" />
                      </div>
                    )}
                  </div>
                  <p className="text-xs text-gray-500 px-4">
                    {qrScanned ? "✓ Fingerprint scanned. Confirm on your phone." : "Scan this code with the Discord mobile app to login instantly."}
                  </p>
                  <button 
                    onClick={() => { setAuthMethod('none'); setQrUrl(null); }}
                    className="text-xs font-black text-red-500 uppercase tracking-widest hover:text-red-400 transition-colors"
                  >
                    Cancel Flow
                  </button>
                </div>
              ) : (
                <div className="py-8 space-y-6">
                  <Smartphone className="w-12 h-12 text-gray-700 mx-auto" />
                  <div>
                    <h3 className="text-xl font-bold mb-2">QR Code Scan</h3>
                    <p className="text-sm text-gray-500 mb-6">Mobile-first secure authentication</p>
                    <button 
                      onClick={handleLoginQR}
                      className="bg-blue-600/10 hover:bg-blue-600/20 text-blue-400 border border-blue-500/30 font-black py-3 px-8 rounded-xl text-xs uppercase tracking-widest transition-all"
                    >
                      Initialize QR Flow
                    </button>
                  </div>
                </div>
              )}
            </div>
          </motion.div>

          <motion.div 
            initial={{ opacity: 0, x: 20 }}
            animate={{ opacity: 1, x: 0 }}
            className="space-y-6"
          >
            <div className="bg-gradient-to-br from-blue-600 to-indigo-700 p-10 rounded-[2.5rem] shadow-2xl shadow-blue-500/20">
              <h3 className="text-2xl font-black mb-4 flex items-center gap-3">
                Official Access
              </h3>
              <p className="text-blue-100/70 text-sm mb-8 leading-relaxed">
                Connect via Discord's official OAuth2 gateway. Standard, secure, and recommended for most users.
              </p>
              <button
                onClick={handleLoginOAuth}
                className="w-full bg-white text-blue-700 font-black py-5 rounded-2xl shadow-xl hover:scale-[1.02] active:scale-95 transition-all text-sm uppercase tracking-widest flex items-center justify-center gap-3"
              >
                <Globe className="w-5 h-5" /> Authorized Login
              </button>
            </div>

            <div className="bg-gray-900/40 border border-gray-800 p-8 rounded-[2rem] backdrop-blur-2xl">
              <h3 className="text-sm font-black text-gray-600 uppercase tracking-[0.2em] mb-6 flex items-center gap-2">
                <Key className="w-4 h-4" /> Advanced
              </h3>
              <form onSubmit={handleLoginToken} className="space-y-4">
                <div className="space-y-2">
                  <p className="text-[10px] text-gray-500 leading-relaxed italic">
                    Paste your user token manually. Use with extreme caution.
                  </p>
                  <input
                    type="password"
                    value={manualToken}
                    onChange={(e) => setManualToken(e.target.value)}
                    placeholder="User Auth Token"
                    className="w-full bg-black/40 border border-gray-800 p-4 rounded-2xl focus:outline-none focus:border-red-500/50 transition-all font-mono text-xs"
                  />
                </div>
                <button
                  type="submit"
                  disabled={!manualToken}
                  className={`w-full py-4 rounded-xl font-black text-xs uppercase tracking-widest transition-all ${manualToken ? 'bg-red-500/10 text-red-500 border border-red-500/20 hover:bg-red-500/20' : 'bg-gray-800 text-gray-600 cursor-not-allowed'}`}
                >
                  Import Token
                </button>
              </form>
            </div>
          </motion.div>
        </div>
      ) : isAuthenticated ? (
        <div className="w-full max-w-5xl space-y-8">
          <motion.div 
            initial={{ opacity: 0, y: 10 }}
            animate={{ opacity: 1, y: 0 }}
            className="flex items-center justify-between bg-gray-900/40 border border-gray-800 p-6 rounded-[2rem] backdrop-blur-md shadow-xl"
          >
            <div className="flex items-center gap-5">
              {user?.avatar ? (
                <img
                  src={`https://cdn.discordapp.com/avatars/${user.id}/${user.avatar}.png`}
                  alt="User Avatar"
                  className="w-16 h-16 rounded-2xl border-2 border-blue-500/30 shadow-2xl"
                />
              ) : (
                <div className="w-16 h-16 rounded-2xl bg-blue-600/20 border border-blue-500/30 flex items-center justify-center font-black text-2xl text-blue-500">
                  {user?.username.charAt(0)}
                </div>
              )}
              <div>
                <h2 className="text-2xl font-black tracking-tight">{user?.username}</h2>
                <div className="flex items-center gap-2 mt-1">
                  <div className="w-2 h-2 bg-green-500 rounded-full animate-pulse" />
                  <p className="text-gray-500 text-[10px] font-black uppercase tracking-[0.2em]">Live Session</p>
                </div>
              </div>
            </div>
            <button
              onClick={setUnauthenticated}
              className="text-gray-500 hover:text-red-500 transition-all text-xs font-black uppercase tracking-widest px-6 py-3 hover:bg-red-500/5 rounded-xl border border-transparent hover:border-red-500/20"
            >
              Terminate
            </button>
          </motion.div>

          <div className="grid grid-cols-1 lg:grid-cols-3 gap-8">
            <div className="lg:col-span-1 space-y-4">
              <h3 className="text-[10px] font-black text-gray-600 uppercase tracking-[0.3em] ml-2">Global Hierarchy</h3>
              <div className="bg-gray-900/40 border border-gray-800 rounded-[2rem] overflow-hidden max-h-[600px] overflow-y-auto custom-scrollbar p-2 space-y-1">
                {guilds?.map((guild) => (
                  <button
                    key={guild.id}
                    onClick={() => handleSelectGuild(guild)}
                    className={`w-full flex items-center gap-4 p-4 rounded-2xl transition-all ${selectedGuild?.id === guild.id ? 'bg-blue-600 text-white shadow-xl shadow-blue-500/20' : 'hover:bg-white/5 text-gray-400'}`}
                  >
                    {guild.icon ? (
                      <img
                        src={`https://cdn.discordapp.com/icons/${guild.id}/${guild.icon}.png`}
                        alt={guild.name}
                        className={`w-10 h-10 rounded-xl shadow-lg transition-transform ${selectedGuild?.id === guild.id ? 'scale-110' : ''}`}
                      />
                    ) : (
                      <div className="w-10 h-10 rounded-xl bg-gray-800 flex items-center justify-center font-bold text-gray-500">
                        {guild.name.charAt(0)}
                      </div>
                    )}
                    <span className="font-bold text-sm truncate">{guild.name}</span>
                  </button>
                ))}
              </div>
            </div>

            <div className="lg:col-span-2 space-y-6">
              {selectedGuild ? (
                <motion.div 
                  initial={{ opacity: 0, x: 20 }}
                  animate={{ opacity: 1, x: 0 }}
                  className="space-y-6"
                >
                  <div className="bg-gray-900/40 border border-gray-800 p-10 rounded-[2.5rem] backdrop-blur-2xl space-y-8 shadow-2xl">
                    <div className="flex items-center justify-between">
                      <div className="space-y-1">
                        <h3 className="text-3xl font-black text-white tracking-tighter">
                          {selectedGuild.name}
                        </h3>
                        {channels && (
                          <p className="text-xs font-bold text-gray-500 uppercase tracking-widest">
                            Scanning {channels.length} data points
                          </p>
                        )}
                      </div>
                    </div>

                    <div className="space-y-4">
                      <label className="text-[10px] font-black text-gray-600 uppercase tracking-[0.2em] ml-1">Temporal Scope</label>
                      <div className="flex gap-2 p-1.5 bg-black/40 rounded-2xl border border-gray-800">
                        {(['24h', '7d', 'all'] as const).map((r) => (
                          <button
                            key={r}
                            onClick={() => setTimeRange(r)}
                            className={`flex-1 py-3 px-4 rounded-xl font-black text-[10px] uppercase tracking-widest transition-all ${timeRange === r ? 'bg-white text-black shadow-lg scale-[1.02]' : 'text-gray-500 hover:text-gray-300'}`}
                          >
                            {r === '24h' ? '24 Hours' : r === '7d' ? '7 Days' : 'Eternal'}
                          </button>
                        ))}
                      </div>
                    </div>

                    <div className="space-y-4">
                      <div className="flex items-center justify-between ml-1">
                        <label className="text-[10px] font-black text-gray-600 uppercase tracking-[0.2em]">Target Buffers</label>
                        <button 
                          onClick={() => setSelectedChannels(new Set(channels?.map(c => c.id)))}
                          className="text-[10px] font-black text-blue-500 hover:text-blue-400 uppercase tracking-widest"
                        >
                          Select All
                        </button>
                      </div>
                      <div className="grid grid-cols-1 md:grid-cols-2 gap-4 max-h-[300px] overflow-y-auto pr-2 custom-scrollbar p-1">
                        {channels?.map((channel) => (
                          <button
                            key={channel.id}
                            onClick={() => toggleChannel(channel.id)}
                            className={`flex items-center justify-between p-4 rounded-2xl border transition-all text-left ${selectedChannels.has(channel.id) ? 'bg-blue-600/10 border-blue-500/50 text-blue-200 shadow-inner' : 'bg-black/20 border-gray-800 text-gray-500 hover:border-gray-700'}`}
                          >
                            <span className="truncate font-bold text-xs tracking-tight">#{channel.name}</span>
                            <div className={`w-4 h-4 rounded-lg flex items-center justify-center border transition-colors ${selectedChannels.has(channel.id) ? 'bg-blue-500 border-blue-500 text-white' : 'border-gray-700'}`}>
                              {selectedChannels.has(channel.id) && <span className="text-[8px] font-black">✓</span>}
                            </div>
                          </button>
                        ))}
                      </div>
                    </div>

                    <div className="pt-6">
                      <button
                        disabled={selectedChannels.size === 0 || isDeleting}
                        onClick={() => setShowConfirmModal(true)}
                        className={`w-full py-6 rounded-[1.5rem] font-black text-sm uppercase tracking-[0.2em] shadow-2xl transition-all ${selectedChannels.size > 0 && !isDeleting ? 'bg-red-600 hover:bg-red-700 text-white shadow-red-500/20 hover:scale-[1.01]' : 'bg-gray-800 text-gray-700 cursor-not-allowed'}`}
                      >
                        {isDeleting ? 'Executing...' : `Purge ${selectedChannels.size} Data Points`}
                      </button>
                    </div>
                  </div>
                </motion.div>
              ) : (
                <div className="h-full flex flex-col items-center justify-center bg-gray-900/20 border border-gray-800 border-dashed rounded-[3rem] p-16 text-center">
                  <div className="w-20 h-20 bg-gray-900/50 rounded-full flex items-center justify-center mb-6 border border-gray-800">
                    <ShieldCheck className="w-10 h-10 text-gray-800" />
                  </div>
                  <h3 className="text-xl font-black tracking-tight text-gray-600 mb-2">Protocol Ready</h3>
                  <p className="text-gray-700 text-xs font-medium max-w-[200px] leading-relaxed">Select a data source from the hierarchy to begin the purge sequence.</p>
                </div>
              )}
            </div>
          </div>
        </div>
      ) : (
        <div className="flex flex-col items-center justify-center h-full space-y-4">
           <div className="w-12 h-12 border-4 border-blue-500/20 border-t-blue-500 rounded-full animate-spin" />
           <p className="text-gray-500 text-xs font-black uppercase tracking-widest">Warming Up...</p>
        </div>
      )}

      {/* Confirmation Modal */}
      <AnimatePresence>
        {showConfirmModal && (
          <motion.div 
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            className="fixed inset-0 bg-black/90 backdrop-blur-xl z-[100] flex items-center justify-center p-6"
          >
            <motion.div 
              initial={{ scale: 0.9, y: 20 }}
              animate={{ scale: 1, y: 0 }}
              className="bg-gray-900 border border-red-500/30 rounded-[3rem] p-12 w-full max-w-lg shadow-2xl space-y-8"
            >
              <div className="text-center space-y-4">
                <div className="w-20 h-20 bg-red-600/10 text-red-600 rounded-[2rem] flex items-center justify-center mx-auto mb-6 border border-red-500/20 shadow-2xl">
                  <ShieldAlert className="w-10 h-10" />
                </div>
                <h2 className="text-4xl font-black tracking-tighter">Confirm Purge</h2>
                <div className="space-y-1 text-gray-400 font-medium">
                  <p>Initializing permanent deletion protocol for</p>
                  <p className="text-white font-black">{selectedChannels.size} target channels</p>
                  <p>in scope <span className="text-white font-black">{selectedGuild?.name}</span>.</p>
                </div>
                <div className="bg-red-500/10 p-4 rounded-2xl border border-red-500/20 inline-block mt-4">
                  <p className="text-[10px] text-red-500 font-black uppercase tracking-[0.2em]">Destructive Action Protocol Active</p>
                </div>
              </div>

              <div className="space-y-3">
                <label className="text-[10px] font-black text-gray-600 uppercase tracking-[0.2em] ml-2">Type <span className="text-red-500 underline">DELETE</span> to authorize</label>
                <input
                  type="text"
                  value={confirmText}
                  onChange={(e) => setConfirmText(e.target.value)}
                  placeholder="AUTHORIZE"
                  className="w-full bg-black/40 border border-gray-800 p-5 rounded-2xl text-center font-black tracking-[0.3em] text-red-500 focus:outline-none focus:border-red-500/50 transition-colors uppercase"
                />
              </div>

              <div className="flex gap-4">
                <button
                  onClick={() => setShowConfirmModal(false)}
                  className="flex-1 py-5 text-gray-500 font-black uppercase tracking-widest text-xs hover:text-white transition-colors"
                >
                  Abort
                </button>
                <button
                  disabled={confirmText !== 'DELETE'}
                  onClick={startDeletion}
                  className={`flex-1 py-5 rounded-2xl font-black text-xs uppercase tracking-widest transition-all ${confirmText === 'DELETE' ? 'bg-red-600 hover:bg-red-700 text-white shadow-xl shadow-red-500/20' : 'bg-gray-800 text-gray-700 cursor-not-allowed'}`}
                >
                  Execute
                </button>
              </div>
            </motion.div>
          </motion.div>
        )}
      </AnimatePresence>

      <AnimatePresence>
        {isDeleting && (
          <motion.div 
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            className="fixed inset-0 bg-gray-950/98 backdrop-blur-3xl z-[110] flex flex-col items-center justify-center p-8 text-center"
          >
            <div className="w-full max-w-2xl space-y-16">
              <div className="space-y-4">
                <motion.div 
                  animate={{ scale: [1, 1.05, 1] }}
                  transition={{ duration: 2, repeat: Infinity }}
                  className="text-6xl font-black tracking-tighter"
                >
                  PURGING...
                </motion.div>
                <div className="flex items-center justify-center gap-3">
                  <div className="w-2 h-2 bg-red-500 rounded-full animate-ping" />
                  <p className="text-red-500 font-black text-xs uppercase tracking-[0.4em]">Active Purge Sequence</p>
                </div>
              </div>

              <div className="space-y-12">
                {deletionProgress ? (
                  <>
                    <div className="space-y-5">
                      <div className="flex justify-between text-[10px] font-black text-gray-500 px-4 uppercase tracking-[0.2em]">
                        <span>Global Purge Level</span>
                        <span className="text-blue-500">Target {deletionProgress.current_channel} / {deletionProgress.total_channels}</span>
                      </div>
                      <div className="w-full h-3 bg-gray-900 rounded-full overflow-hidden border border-gray-800 p-0.5">
                        <motion.div 
                          initial={{ width: 0 }}
                          animate={{ width: `${(deletionProgress.current_channel / deletionProgress.total_channels) * 100}%` }}
                          className="h-full bg-gradient-to-r from-blue-600 via-purple-600 to-red-600 rounded-full shadow-[0_0_20px_rgba(59,130,246,0.3)]"
                        />
                      </div>
                    </div>

                    <div className="grid grid-cols-2 gap-10">
                      <div className="bg-gray-900/50 p-8 rounded-3xl border border-gray-800 shadow-inner">
                        <p className="text-[10px] font-black text-gray-600 uppercase tracking-widest mb-2">Phase</p>
                        <p className="text-2xl font-black text-blue-500 uppercase tracking-tighter">{deletionProgress.status}</p>
                      </div>
                      <div className="bg-gray-900/50 p-8 rounded-3xl border border-gray-800 shadow-inner">
                        <p className="text-[10px] font-black text-gray-600 uppercase tracking-widest mb-2">Eliminated</p>
                        <p className="text-2xl font-black text-red-500 tracking-tighter">{deletionProgress.deleted_count}</p>
                      </div>
                    </div>

                    <div className="space-y-2">
                       <p className="text-gray-600 text-[10px] font-black uppercase tracking-widest">Current Buffer</p>
                       <p className="text-white font-mono text-sm font-bold">
                        #{channels?.find(c => c.id === deletionProgress.channel_id)?.name || '0xUNKNOWN'}
                       </p>
                    </div>
                  </>
                ) : (
                  <div className="space-y-8">
                    <div className="w-20 h-20 border-4 border-blue-500/10 border-t-blue-500 rounded-full animate-spin mx-auto shadow-2xl shadow-blue-500/20" />
                    <p className="text-gray-500 text-xs font-black uppercase tracking-[0.2em] animate-pulse">Syncing with Discord Gateway...</p>
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
