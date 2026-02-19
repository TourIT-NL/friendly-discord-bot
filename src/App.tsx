import React, { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { useAuthStore } from './store/authStore';
import { motion, AnimatePresence } from 'framer-motion';
import { QRCodeSVG } from 'qrcode.react';
import * as Tooltip from '@radix-ui/react-tooltip';
import { 
  Monitor, Smartphone, Key, Globe, ShieldCheck, ShieldAlert, 
  CheckCircle2, XCircle, HelpCircle, ArrowLeft, Info, 
  Trash2, Server, Hash, Clock
} from 'lucide-react';

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
  status: string;
}

const HelpMarker = ({ content }: { content: React.ReactNode }) => (
  <Tooltip.Provider delayDuration={100}>
    <Tooltip.Root>
      <Tooltip.Trigger asChild>
        <button className="text-gray-500 hover:text-blue-400 transition-colors p-1" type="button">
          <HelpCircle className="w-4 h-4" />
        </button>
      </Tooltip.Trigger>
      <Tooltip.Portal>
        <Tooltip.Content 
          className="bg-gray-900 border border-gray-800 p-4 rounded-xl shadow-2xl max-w-xs text-sm text-gray-300 leading-relaxed z-[200] animate-in fade-in zoom-in-95"
          sideOffset={5}
        >
          {content}
          <Tooltip.Arrow className="fill-gray-900" />
        </Tooltip.Content>
      </Tooltip.Portal>
    </Tooltip.Root>
  </Tooltip.Provider>
);

function App() {
  const { 
    isAuthenticated, needsCredentials, user, guilds, isLoading, error, 
    setAuthenticated, setUnauthenticated, setLoading, setError, setGuilds, setNeedsCredentials 
  } = useAuthStore();
  
  const [selectedGuild, setSelectedGuild] = useState<Guild | null>(null);
  const [channels, setChannels] = useState<Channel[] | null>(null);
  const [selectedChannels, setSelectedChannels] = useState<Set<string>>(new Set());
  const [isDeleting, setIsDeleting] = useState(false);
  const [deletionProgress, setDeletionProgress] = useState<DeletionProgress | null>(null);
  const [showConfirmModal, setShowConfirmModal] = useState(false);
  const [confirmText, setConfirmText] = useState('');
  const [timeRange, setTimeRange] = useState<'24h' | '7d' | 'all'>('all');

  const [clientId, setClientId] = useState('');
  const [clientSecret, setClientSecret] = useState('');

  const [authMethod, setAuthMethod] = useState<'none' | 'oauth' | 'qr' | 'token' | 'rpc'>('none');
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
      setError(err.message || "Failed to fetch guilds.");
    } finally {
      setLoading(false);
    }
  };

  const checkDiscordStatus = async () => {
    try {
      const status: DiscordStatus = await invoke('check_discord_status');
      setDiscordStatus(status);
    } catch (err) {}
  };

  useEffect(() => {
    checkDiscordStatus();
    const interval = setInterval(checkDiscordStatus, 5000);
    return () => clearInterval(interval);
  }, []);

  useEffect(() => {
    const unlisteners: Promise<() => void>[] = [
      listen('auth_started', () => { setLoading(true); setError(null); }),
      listen('auth_success', (event) => {
        setAuthenticated(event.payload as DiscordUser);
        setAuthMethod('none');
        setQrUrl(null);
        setQrScanned(false);
        fetchGuilds();
      }),
      listen<string>('qr_code_ready', (event) => { setQrUrl(event.payload); setLoading(false); }),
      listen('qr_scanned', () => setQrScanned(true)),
      listen('qr_cancelled', () => {
        setAuthMethod('none'); setQrUrl(null); setQrScanned(false);
        setError("QR Login timed out or was cancelled.");
      }),
      listen('deletion_progress', (event) => setDeletionProgress(event.payload as DeletionProgress)),
      listen('deletion_complete', () => { setIsDeleting(false); setDeletionProgress(null); }),
    ];
    return () => {
      unlisteners.forEach(async (u) => (await u)());
    };
  }, []);

  const handleLoginOAuth = async () => {
    setLoading(true); setError(null);
    try {
      await invoke('start_oauth_flow');
    } catch (err: any) {
      if (err.error_code === 'credentials_missing') setNeedsCredentials(true);
      else setError(err.user_message || "OAuth login failed.");
      setLoading(false);
    }
  };

  const handleLoginQR = async () => {
    setAuthMethod('qr'); setLoading(true); setError(null);
    try {
      await invoke('start_qr_login_flow');
    } catch (err: any) {
      setError(err.user_message || "QR initialization failed.");
      setLoading(false); setAuthMethod('none');
    }
  };

  const handleLoginRPC = async () => {
    setAuthMethod('rpc'); setLoading(true); setError(null);
    try {
      await invoke('login_with_rpc');
    } catch (err: any) {
      setError(err.user_message || "Instant Link failed.");
      setAuthMethod('none'); setLoading(false);
    }
  };

  const handleLoginToken = async (e: React.FormEvent) => {
    e.preventDefault(); setLoading(true); setError(null);
    try {
      await invoke('login_with_user_token', { token: manualToken });
    } catch (err: any) {
      setError(err.user_message || "Token login failed.");
      setLoading(false);
    }
  };

  const handleSaveCredentials = async (e: React.FormEvent) => {
    e.preventDefault(); setLoading(true); setError(null);
    try {
      await invoke('save_discord_credentials', { clientId, clientSecret });
      setNeedsCredentials(false);
      // Automatically trigger login after saving
      setTimeout(() => handleLoginOAuth(), 100);
    } catch (err: any) {
      setError("Failed to save credentials.");
      setLoading(false);
    }
  };

  const handleSelectGuild = async (guild: Guild) => {
    setSelectedGuild(guild); setChannels(null); setSelectedChannels(new Set());
    try {
      setLoading(true);
      const fetchedChannels: Channel[] = await invoke('fetch_channels', { guildId: guild.id });
      setChannels(fetchedChannels);
    } catch (err: any) {
      setError("Failed to fetch channels.");
    } finally {
      setLoading(false);
    }
  };

  const toggleChannel = (id: string) => {
    const next = new Set(selectedChannels);
    if (next.has(id)) next.delete(id); else next.add(id);
    setSelectedChannels(next);
  };

  const startDeletion = async () => {
    if (confirmText !== 'DELETE') return;
    setShowConfirmModal(false); setIsDeleting(true);
    const now = Date.now();
    let startTime: number | undefined;
    if (timeRange === '24h') startTime = now - 86400000;
    else if (timeRange === '7d') startTime = now - 604800000;
    try {
      await invoke('bulk_delete_messages', { channelIds: Array.from(selectedChannels), startTime, endTime: undefined });
    } catch (err: any) {
      setError(err.user_message || "Purge failed.");
      setIsDeleting(false);
    }
  };

  if (needsCredentials) {
    return (
      <div className="min-h-screen bg-gray-950 text-white flex items-center justify-center p-6">
        <div className="w-full max-w-md bg-gray-900 border border-gray-800 rounded-3xl p-10 shadow-2xl">
          <div className="flex items-center gap-3 mb-8">
            <button onClick={() => setNeedsCredentials(false)} className="p-2 bg-gray-800 rounded-xl hover:bg-gray-700 transition-colors text-gray-400 hover:text-white"><ArrowLeft className="w-5 h-5" /></button>
            <h2 className="text-3xl font-black tracking-tight">API SETUP</h2>
          </div>
          <form onSubmit={handleSaveCredentials} className="space-y-6">
            <div className="space-y-2">
              <label className="text-[10px] font-black text-gray-500 uppercase tracking-widest ml-2">Client ID</label>
              <input type="text" required value={clientId} onChange={e => setClientId(e.target.value)} className="w-full bg-black border border-gray-800 p-4 rounded-2xl focus:border-blue-500 outline-none font-mono text-sm" placeholder="Application ID" />
            </div>
            <div className="space-y-2">
              <label className="text-[10px] font-black text-gray-500 uppercase tracking-widest ml-2">Client Secret</label>
              <input type="password" required value={clientSecret} onChange={e => setClientSecret(e.target.value)} className="w-full bg-black border border-gray-800 p-4 rounded-2xl focus:border-blue-500 outline-none font-mono text-sm" placeholder="••••••••" />
            </div>
            <button type="submit" className="w-full bg-blue-600 hover:bg-blue-700 py-5 rounded-2xl font-black text-xs uppercase tracking-widest transition-all shadow-xl shadow-blue-900/20">Save & Authorize</button>
            <div className="flex items-center gap-3 justify-center mt-8 p-4 bg-black/40 rounded-xl border border-gray-800">
              <Info className="w-4 h-4 text-blue-500" />
              <p className="text-[10px] text-gray-500 font-bold uppercase tracking-widest">
                Setup available in <a href="https://discord.com/developers/applications" target="_blank" rel="noopener noreferrer" className="text-blue-500 hover:text-blue-400 font-bold">Discord Dev Portal</a>
              </p>
            </div>
          </form>
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-gray-950 text-gray-100 font-sans p-10 selection:bg-blue-500/30">
      <header className="max-w-6xl mx-auto mb-16 flex items-center justify-between">
        <div>
          <h1 className="text-4xl font-black tracking-tighter text-white italic">DISCORD PURGE</h1>
          <div className="flex items-center gap-3 mt-1">
            <div className="h-px w-6 bg-blue-600" />
            <p className="text-[10px] text-gray-500 font-black tracking-[0.4em] uppercase">Privacy Enforcement</p>
          </div>
        </div>
        {isAuthenticated && (
          <div className="flex items-center gap-5 bg-gray-900 border border-gray-800 p-3 pr-6 rounded-2xl shadow-xl">
            {user?.avatar ? <img src={`https://cdn.discordapp.com/avatars/${user.id}/${user.avatar}.png`} className="w-10 h-10 rounded-xl shadow-lg" /> : <div className="w-10 h-10 rounded-xl bg-blue-600 flex items-center justify-center font-black text-lg">{user?.username[0]}</div>}
            <div>
              <p className="text-sm font-black">{user?.username}</p>
              <p className="text-[10px] text-green-500 font-bold uppercase tracking-widest">Authorized</p>
            </div>
            <button onClick={setUnauthenticated} className="p-2 hover:bg-red-500/10 rounded-lg text-gray-500 hover:text-red-500 transition-colors ml-2"><XCircle className="w-5 h-5" /></button>
          </div>
        )}
      </header>

      <main className="max-w-6xl mx-auto">
        <AnimatePresence mode="wait">
          {!isAuthenticated ? (
            <motion.div initial={{ opacity: 0, y: 10 }} animate={{ opacity: 1, y: 0 }} exit={{ opacity: 0, y: -10 }}>
              {authMethod === 'none' ? (
                <div className="grid grid-cols-1 md:grid-cols-2 gap-10">
                  <section className="space-y-8">
                    <div className="bg-gray-900/50 border border-gray-800 rounded-[2.5rem] p-10 backdrop-blur-3xl shadow-2xl">
                      <div className="flex items-center justify-between mb-10">
                        <h3 className="text-xs font-black text-gray-500 uppercase tracking-[0.3em] flex items-center gap-3"><Monitor className="w-4 h-4 text-blue-500" /> Environment</h3>
                        <HelpMarker content="We scan for a running Discord client to enable the Instant Link handshake." />
                      </div>
                      <div className="space-y-5">
                        <div className="flex items-center justify-between p-5 bg-black/40 rounded-2xl border border-white/5 shadow-inner">
                          <span className="text-sm font-bold text-gray-400">Desktop Client</span>
                          <span className={`text-[10px] font-black px-3 py-1 rounded-full ${discordStatus?.is_running ? 'bg-green-500/10 text-green-500' : 'bg-gray-800 text-gray-600'}`}>{discordStatus?.is_running ? 'ACTIVE' : 'OFFLINE'}</span>
                        </div>
                        <button 
                          onClick={handleLoginRPC}
                          disabled={!discordStatus?.rpc_available}
                          className={`w-full flex items-center justify-between p-5 bg-black/40 rounded-2xl border transition-all ${discordStatus?.rpc_available ? 'border-blue-500/50 hover:bg-blue-500/10 hover:scale-[1.02]' : 'border-white/5 opacity-50'}`}
                        >
                          <span className="text-sm font-black text-blue-400">Instant Link</span>
                          <div className="flex items-center gap-2">
                            <ShieldCheck className={`w-4 h-4 ${discordStatus?.rpc_available ? 'text-blue-500' : 'text-gray-700'}`} />
                            <span className="text-[10px] font-black">{discordStatus?.rpc_available ? 'READY' : 'UNAVAILABLE'}</span>
                          </div>
                        </button>
                      </div>
                    </div>

                    <div className="bg-gray-900/50 border border-gray-800 rounded-[2.5rem] p-12 text-center group hover:border-blue-500/30 transition-all shadow-2xl backdrop-blur-3xl">
                      <Smartphone className="w-12 h-12 text-gray-700 mx-auto mb-6 group-hover:text-blue-500 transition-colors" />
                      <h3 className="text-2xl font-black mb-2 tracking-tight">QR CODE</h3>
                      <p className="text-gray-500 text-sm mb-10 px-6">Scan with the Discord mobile app for passwordless auth.</p>
                      <button onClick={handleLoginQR} className="w-full bg-blue-600/10 text-blue-400 border border-blue-500/30 py-5 rounded-2xl font-black text-xs uppercase tracking-widest hover:bg-blue-600 hover:text-white transition-all shadow-lg hover:shadow-blue-500/20">Initialize QR Flow</button>
                    </div>
                  </section>

                  <section className="space-y-8">
                    <div className="bg-gradient-to-br from-blue-600 to-indigo-900 rounded-[3rem] p-12 shadow-2xl shadow-blue-900/30 relative overflow-hidden group">
                      <Globe className="absolute -right-6 -bottom-6 w-40 h-40 text-white/10 group-hover:rotate-12 transition-transform duration-700" />
                      <div className="relative z-10">
                        <h3 className="text-3xl font-black mb-3 tracking-tighter italic">OFFICIAL GATE</h3>
                        <p className="text-blue-100/70 text-sm mb-12 leading-relaxed max-w-xs font-medium">Standard OAuth2 flow. Secure, authorized, and recommended for most users.</p>
                        <button onClick={handleLoginOAuth} className="w-full bg-white text-blue-700 py-6 rounded-[1.5rem] font-black text-xs uppercase tracking-[0.2em] hover:scale-[1.03] active:scale-95 transition-all shadow-2xl">Start Authorization</button>
                      </div>
                    </div>

                    <button onClick={() => setAuthMethod('token')} className="w-full bg-gray-900/50 border border-gray-800 p-10 rounded-[2.5rem] flex items-center justify-between hover:bg-gray-800 transition-all group shadow-2xl backdrop-blur-3xl">
                      <div className="flex items-center gap-6">
                        <div className="bg-red-500/10 p-4 rounded-2xl text-red-500 group-hover:scale-110 transition-transform"><Key className="w-6 h-6" /></div>
                        <div className="text-left">
                          <h4 className="text-lg font-black tracking-tight">MANUAL TOKEN</h4>
                          <p className="text-[10px] text-gray-500 font-bold uppercase tracking-widest mt-1">Advanced Bypass</p>
                        </div>
                      </div>
                      <HelpMarker content="Input your account token directly. WARNING: This bypasses all security checks. Only for expert use." />
                    </button>
                  </section>
                </div>
              ) : authMethod === 'qr' ? (
                <div className="max-w-md mx-auto text-center bg-gray-900/50 border border-gray-800 p-12 rounded-[3.5rem] shadow-2xl backdrop-blur-3xl">
                  <button onClick={() => { setAuthMethod('none'); setQrUrl(null); setQrScanned(false); }} className="flex items-center gap-2 text-[10px] font-black text-gray-500 hover:text-white uppercase tracking-widest mb-10 transition-colors"><ArrowLeft className="w-4 h-4" /> Return to Menu</button>
                  <div className="bg-white p-6 rounded-[2.5rem] inline-block mb-10 shadow-[0_0_50px_rgba(255,255,255,0.1)]">
                    {qrUrl ? <QRCodeSVG value={qrUrl} size={220} level="H" includeMargin={true} /> : <div className="w-[220px] h-[220px] flex items-center justify-center bg-gray-50 rounded-2xl"><div className="w-10 h-10 border-4 border-blue-500 border-t-transparent rounded-full animate-spin" /></div>}
                  </div>
                  <p className="text-sm font-bold text-gray-400 leading-relaxed px-6">
                    {qrScanned ? <span className="text-green-500 flex items-center justify-center gap-2 animate-pulse"><CheckCircle2 className="w-5 h-5" /> SCANNED! CONFIRM ON MOBILE</span> : "Open Discord on your mobile device, navigate to Settings > Scan QR Code."}
                  </p>
                </div>
              ) : (
                <div className="max-w-lg mx-auto bg-gray-900/50 border border-gray-800 p-12 rounded-[3.5rem] shadow-2xl backdrop-blur-3xl">
                  <button onClick={() => setAuthMethod('none')} className="flex items-center gap-2 text-[10px] font-black text-gray-500 hover:text-white uppercase tracking-widest mb-10 transition-colors"><ArrowLeft className="w-4 h-4" /> Return to Menu</button>
                  <div className="flex items-center justify-between mb-8 px-2">
                    <h3 className="text-2xl font-black tracking-tighter">TOKEN ENTRY</h3>
                    <HelpMarker content={
                      <div className="space-y-4">
                        <p className="font-black text-red-500 uppercase tracking-widest underline">Security Warning</p>
                        <p className="text-xs">Your token gives FULL access. Never share it.</p>
                        <hr className="border-gray-800" />
                        <p className="font-bold">Instructions:</p>
                        <ol className="list-decimal list-inside space-y-2 text-[11px]">
                          <li>Browser &gt; <code className="bg-black p-1 rounded">F12</code></li>
                          <li>Tab: <span className="text-blue-400 font-bold">Network</span></li>
                          <li>Filter: <code className="bg-black p-1 rounded">/api</code></li>
                          <li>Select any request</li>
                          <li>Header: <span className="text-blue-400 font-bold">authorization</span></li>
                        </ol>
                      </div>
                    } />
                  </div>
                  <form onSubmit={handleLoginToken} className="space-y-8">
                    <input type="password" value={manualToken} onChange={e => setManualToken(e.target.value)} className="w-full bg-black border border-gray-800 p-5 rounded-2xl focus:border-red-500 outline-none font-mono text-xs shadow-inner" placeholder="NJAY..." />
                    <button type="submit" disabled={!manualToken} className="w-full bg-red-600 hover:bg-red-700 py-5 rounded-2xl font-black text-xs uppercase tracking-widest transition-all disabled:opacity-50 shadow-xl shadow-red-900/20">Establish Connection</button>
                  </form>
                </div>
              )}
            </motion.div>
          ) : (
            <motion.div initial={{ opacity: 0 }} animate={{ opacity: 1 }} className="grid grid-cols-1 lg:grid-cols-12 gap-10">
              <aside className="lg:col-span-4 space-y-6">
                <h3 className="text-[10px] font-black text-gray-600 uppercase tracking-[0.4em] ml-4 flex items-center gap-3"><Server className="w-3 h-3" /> Target Sources</h3>
                <div className="bg-gray-900/40 border border-gray-800 rounded-[3rem] overflow-hidden max-h-[600px] overflow-y-auto custom-scrollbar p-3 space-y-2 shadow-inner">
                  {guilds?.map(g => (
                    <button key={g.id} onClick={() => handleSelectGuild(g)} className={`w-full flex items-center gap-5 p-5 rounded-[2rem] transition-all relative overflow-hidden group ${selectedGuild?.id === g.id ? 'bg-blue-600 text-white shadow-2xl' : 'hover:bg-white/5 text-gray-400'}`}>
                      {g.icon ? <img src={`https://cdn.discordapp.com/icons/${g.id}/${g.icon}.png`} className="w-12 h-12 rounded-2xl shadow-xl transition-transform group-hover:scale-110" /> : <div className="w-12 h-12 rounded-2xl bg-gray-800 flex items-center justify-center font-black text-lg">{g.name[0]}</div>}
                      <span className="text-sm font-black tracking-tight truncate">{g.name}</span>
                      {selectedGuild?.id === g.id && <div className="absolute right-6 w-2 h-2 bg-white rounded-full shadow-[0_0_10px_white]" />}
                    </button>
                  ))}
                </div>
              </aside>

              <div className="lg:col-span-8">
                {selectedGuild ? (
                  <div className="bg-gray-900/40 border border-gray-800 p-12 rounded-[3.5rem] space-y-10 shadow-2xl backdrop-blur-3xl relative overflow-hidden">
                    <div className="absolute top-0 right-0 p-10 opacity-5"><Trash2 className="w-32 h-32 rotate-12" /></div>
                    <div className="relative z-10">
                      <h3 className="text-4xl font-black text-white tracking-tighter italic uppercase">{selectedGuild.name}</h3>
                      <p className="text-[10px] text-blue-500 font-bold uppercase tracking-[0.3em] mt-2">Active Buffer Mapping</p>
                    </div>
                    
                    <div className="space-y-6 relative z-10">
                      <label className="text-[10px] font-black text-gray-600 uppercase tracking-[0.3em] ml-2 flex items-center gap-3"><Clock className="w-3 h-3" /> Temporal Scope</label>
                      <div className="flex gap-3 p-2 bg-black/40 rounded-[1.5rem] border border-gray-800 shadow-inner">
                        {(['24h', '7d', 'all'] as const).map(r => (
                          <button key={r} onClick={() => setTimeRange(r)} className={`flex-1 py-4 rounded-[1.2rem] text-[10px] font-black uppercase tracking-widest transition-all ${timeRange === r ? 'bg-white text-black shadow-xl' : 'text-gray-500 hover:text-gray-300'}`}>{r === '24h' ? '24 HOURS' : r === '7d' ? '7 DAYS' : 'ETERNAL'}</button>
                        ))}
                      </div>
                    </div>

                    <div className="space-y-6 relative z-10">
                      <div className="flex items-center justify-between px-2">
                        <label className="text-[10px] font-black text-gray-600 uppercase tracking-[0.3em] flex items-center gap-3"><Hash className="w-3 h-3" /> Target Buffers</label>
                        <button onClick={() => setSelectedChannels(new Set(channels?.map(c => c.id)))} className="text-[10px] font-black text-blue-500 hover:text-blue-400 uppercase tracking-widest underline decoration-2 underline-offset-4">Select All</button>
                      </div>
                      <div className="grid grid-cols-1 md:grid-cols-2 gap-4 max-h-[350px] overflow-y-auto pr-3 custom-scrollbar p-1">
                        {channels?.map(c => (
                          <button key={c.id} onClick={() => toggleChannel(c.id)} className={`flex items-center justify-between p-5 rounded-[2rem] border-2 transition-all text-left ${selectedChannels.has(c.id) ? 'bg-blue-600/10 border-blue-500 shadow-xl text-blue-100' : 'bg-black/20 border-gray-800 text-gray-500 hover:border-gray-700'}`}>
                            <span className="truncate font-black text-xs tracking-tight">#{c.name}</span>
                            <div className={`w-5 h-5 rounded-lg border-2 flex items-center justify-center transition-all ${selectedChannels.has(c.id) ? 'bg-blue-500 border-blue-500 scale-110 shadow-[0_0_15px_rgba(59,130,246,0.5)]' : 'border-gray-800'}`}>{selectedChannels.has(c.id) && <CheckCircle2 className="w-3 h-3 text-white" />}</div>
                          </button>
                        ))}
                      </div>
                    </div>

                    <button disabled={selectedChannels.size === 0 || isDeleting} onClick={() => setShowConfirmModal(true)} className="w-full bg-red-600 hover:bg-red-700 py-7 rounded-[2.2rem] font-black text-xs uppercase tracking-[0.3em] shadow-2xl shadow-red-900/20 transition-all hover:scale-[1.01] active:scale-95 disabled:opacity-50 relative z-10">Initialize Purge Sequence ({selectedChannels.size})</button>
                  </div>
                ) : (
                  <div className="h-full min-h-[500px] flex flex-col items-center justify-center border-2 border-dashed border-gray-800 rounded-[4rem] p-20 text-center opacity-40">
                    <ShieldCheck className="w-16 h-16 mb-6" />
                    <p className="font-black uppercase tracking-[0.4em] text-[10px]">Awaiting Data Stream Selection</p>
                  </div>
                )}
              </div>
            </motion.div>
          )}
        </AnimatePresence>
      </main>

      {/* Confirmation Modal */}
      <AnimatePresence>
        {showConfirmModal && (
          <div className="fixed inset-0 bg-black/95 backdrop-blur-2xl flex items-center justify-center p-6 z-[300]">
            <motion.div initial={{ scale: 0.9, y: 20 }} animate={{ scale: 1, y: 0 }} className="bg-gray-900 border border-red-500/30 rounded-[4rem] p-16 max-w-xl w-full space-y-10 text-center shadow-2xl relative overflow-hidden">
              <div className="absolute inset-0 bg-gradient-to-br from-red-500/5 to-transparent pointer-events-none" />
              <ShieldAlert className="w-16 h-16 text-red-600 mx-auto mb-4 drop-shadow-[0_0_15px_rgba(220,38,38,0.5)]" />
              <h2 className="text-5xl font-black tracking-tighter">FINAL VERIFICATION</h2>
              <p className="text-gray-400 font-bold text-sm uppercase tracking-widest leading-relaxed">
                Confirming destructive protocol for <span className="text-white font-black">{selectedChannels.size} target channels</span> in <span className="text-white font-black">{selectedGuild?.name}</span>.
              </p>
              <div className="space-y-4">
                <label className="text-[10px] font-black text-gray-600 uppercase tracking-[0.4em]">Authorized Signature (<span className="text-red-500 underline">DELETE</span>)</label>
                <input type="text" value={confirmText} onChange={e => setConfirmText(e.target.value.toUpperCase())} className="w-full bg-black/60 border border-gray-800 p-6 rounded-[2rem] text-center text-red-500 font-black tracking-[0.6em] outline-none text-xl shadow-inner focus:border-red-500/50 transition-colors" placeholder="AUTHORIZE" />
              </div>
              <div className="flex gap-6">
                <button onClick={() => setShowConfirmModal(false)} className="flex-1 py-6 text-gray-500 font-black uppercase text-[10px] tracking-widest border border-gray-800 rounded-[1.5rem] hover:bg-white/5 transition-all">Abort Protocol</button>
                <button disabled={confirmText !== 'DELETE'} onClick={startDeletion} className={`flex-1 py-6 rounded-[1.5rem] font-black text-[10px] tracking-widest uppercase transition-all shadow-2xl ${confirmText === 'DELETE' ? 'bg-red-600 hover:bg-red-700 text-white shadow-red-900/40 scale-105' : 'bg-gray-800 text-gray-700 cursor-not-allowed'}`}>Execute Purge</button>
              </div>
            </motion.div>
          </div>
        )}
      </AnimatePresence>

      <AnimatePresence>
        {isDeleting && (
          <div className="fixed inset-0 bg-gray-950/99 backdrop-blur-[100px] flex flex-col items-center justify-center p-10 text-center z-[400]">
            <motion.h2 animate={{ scale: [1, 1.02, 1], opacity: [0.7, 1, 0.7] }} transition={{ repeat: Infinity, duration: 2 }} className="text-7xl font-black italic tracking-tighter mb-20">PURGING...</motion.h2>
            {deletionProgress ? (
              <div className="w-full max-w-2xl space-y-12 relative">
                <div className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-96 h-96 bg-blue-600/10 rounded-full blur-[120px] -z-10" />
                <div className="space-y-6">
                  <div className="flex justify-between text-[10px] font-black text-gray-500 px-6 uppercase tracking-[0.4em]">
                    <span>Global Saturation</span>
                    <span className="text-blue-500">Target {deletionProgress.current_channel} / {deletionProgress.total_channels}</span>
                  </div>
                  <div className="w-full h-4 bg-gray-900 rounded-full overflow-hidden border border-white/5 p-1 shadow-inner">
                    <motion.div animate={{ width: `${(deletionProgress.current_channel / deletionProgress.total_channels) * 100}%` }} className="h-full bg-gradient-to-r from-blue-600 via-purple-600 to-red-600 shadow-[0_0_25px_blue]" />
                  </div>
                </div>
                <div className="grid grid-cols-2 gap-10">
                  <div className="bg-black/40 p-10 rounded-[2.5rem] border border-white/5 shadow-2xl backdrop-blur-3xl">
                    <p className="text-[10px] text-gray-600 font-black uppercase tracking-[0.4em] mb-3 text-left">Status</p>
                    <p className="text-3xl font-black text-blue-500 uppercase tracking-tighter italic text-left">{deletionProgress.status}</p>
                  </div>
                  <div className="bg-black/40 p-10 rounded-[2.5rem] border border-white/5 shadow-2xl backdrop-blur-3xl">
                    <p className="text-[10px] text-gray-600 font-black uppercase tracking-[0.4em] mb-3 text-left">Eliminated</p>
                    <p className="text-3xl font-black text-red-500 tracking-tighter italic text-left">{deletionProgress.deleted_count}</p>
                  </div>
                </div>
                <div className="p-4 bg-white/5 rounded-2xl border border-white/5 inline-block">
                  <p className="text-[10px] text-gray-500 font-black uppercase tracking-[0.5em]">Scanning: #{channels?.find(c => c.id === deletionProgress.channel_id)?.name || '0xUNKNOWN'}</p>
                </div>
              </div>
            ) : <div className="w-24 h-24 border-4 border-blue-500 border-t-transparent rounded-[2rem] animate-spin shadow-[0_0_50px_blue]" />}
          </div>
        )}
      </AnimatePresence>
    </div>
  );
}

export default App;
