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
  Trash2, Server, Hash, Clock, LogOut, Settings
} from 'lucide-react';

interface DiscordUser { id: string; username: string; avatar?: string; email?: string; }
interface DiscordStatus { is_running: boolean; rpc_available: boolean; browser_detected: boolean; }
interface Guild { id: string; name: string; icon?: string; }
interface Channel { id: string; name: string; }
interface DeletionProgress { current_channel: number; total_channels: number; channel_id: string; deleted_count: number; status: string; }
interface LeaveProgress { current: number; total: number; guild_id: string; status: string; }

const HelpMarker = ({ content }: { content: React.ReactNode }) => (
  <Tooltip.Provider delayDuration={100}>
    <Tooltip.Root>
      <Tooltip.Trigger asChild>
        <button className="text-gray-600 hover:text-blue-500 transition-colors p-1 flex items-center justify-center focus:outline-none" type="button">
          <HelpCircle className="w-4 h-4" />
        </button>
      </Tooltip.Trigger>
      <Tooltip.Portal>
        <Tooltip.Content className="bg-gray-900 border border-gray-800 p-4 rounded-xl shadow-2xl max-w-xs text-sm text-gray-300 z-[200] animate-in fade-in" sideOffset={5}>
          {content}<Tooltip.Arrow className="fill-gray-900" />
        </Tooltip.Content>
      </Tooltip.Portal>
    </Tooltip.Root>
  </Tooltip.Provider>
);

function App() {
  const { 
    isAuthenticated, needsCredentials, appMode, user, guilds, isLoading, error, 
    setAuthenticated, setUnauthenticated, setLoading, setError, setGuilds, setNeedsCredentials, setAppMode 
  } = useAuthStore();
  
  const [selectedGuild, setSelectedGuild] = useState<Guild | null>(null);
  const [channels, setChannels] = useState<Channel[] | null>(null);
  const [selectedChannels, setSelectedChannels] = useState<Set<string>>(new Set());
  const [selectedGuildsToLeave, setSelectedGuildsToLeave] = useState<Set<string>>(new Set());
  
  const [isProcessing, setIsProcessing] = useState(false);
  const [deletionProgress, setDeletionProgress] = useState<DeletionProgress | null>(null);
  const [leaveProgress, setLeaveProgress] = useState<LeaveProgress | null>(null);
  
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
    setLoading(true);
    try {
      const fetched: Guild[] = await invoke('fetch_guilds');
      setGuilds(fetched);
    } catch (err: any) {
      setError(err.user_message || "Failed to fetch guilds.");
    } finally { setLoading(false); }
  };

  const checkDiscordStatus = async () => {
    try {
      const status: DiscordStatus = await invoke('check_discord_status');
      setDiscordStatus(status);
    } catch (err) {}
  };

  const tryAutoLogin = async () => {
    try {
      await invoke('get_current_user');
    } catch (err) {
      console.log("No existing session found or auto-login failed.");
    }
  };

  useEffect(() => {
    tryAutoLogin();
    checkDiscordStatus();
    const interval = setInterval(checkDiscordStatus, 5000);
    return () => clearInterval(interval);
  }, []);

  useEffect(() => {
    let unlisteners: any[] = [];
    const setup = async () => {
      unlisteners.push(await listen('auth_started', () => { setLoading(true); setError(null); }));
      unlisteners.push(await listen('auth_success', (event) => {
        setAuthenticated(event.payload as DiscordUser);
        setAuthMethod('none'); setQrUrl(null); setQrScanned(false);
        fetchGuilds();
      }));
      unlisteners.push(await listen<string>('qr_code_ready', (event) => { setQrUrl(event.payload); setLoading(false); }));
      unlisteners.push(await listen('qr_scanned', () => setQrScanned(true)));
      unlisteners.push(await listen('qr_cancelled', () => {
        setAuthMethod('none'); setQrUrl(null); setQrScanned(false);
        setError("QR Handshake was interrupted.");
      }));
      unlisteners.push(await listen('deletion_progress', (event) => setDeletionProgress(event.payload as DeletionProgress)));
      unlisteners.push(await listen('deletion_complete', () => { 
        setIsProcessing(false); setDeletionProgress(null); 
        fetchGuilds(); // Refresh guilds after deletion
      }));
      unlisteners.push(await listen('leave_progress', (event) => setLeaveProgress(event.payload as LeaveProgress)));
      unlisteners.push(await listen('leave_complete', () => {
        setIsProcessing(false); setLeaveProgress(null);
        fetchGuilds(); // Refresh guilds after leaving
      }));
    };
    setup();
    return () => unlisteners.forEach(u => u && u());
  }, []);

  const handleLoginOAuth = async () => {
    setLoading(true); setError(null);
    try { await invoke('start_oauth_flow'); } catch (err: any) {
      if (err.error_code === 'credentials_missing') setNeedsCredentials(true);
      else setError(err.user_message || "OAuth Link failed.");
      setLoading(false);
    }
  };

  const handleLoginQR = async () => {
    setAuthMethod('qr'); setLoading(true); setError(null);
    try { await invoke('start_qr_login_flow'); } catch (err: any) {
      setError(err.user_message || "QR Handshake failed.");
      setLoading(false); setAuthMethod('none');
    }
  };

  const handleLoginRPC = async () => {
    setAuthMethod('rpc'); setLoading(true); setError(null);
    try { await invoke('login_with_rpc'); } catch (err: any) {
      setError(err.user_message || "Instant Link failed.");
      setAuthMethod('none'); setLoading(false);
    }
  };

  const handleLoginToken = async (e: React.FormEvent) => {
    e.preventDefault(); setLoading(true); setError(null);
    try { 
      const cleanToken = manualToken.trim().replace(/^Bearer\s+/i, '').replace(/^"|"$/g, '');
      await invoke('login_with_user_token', { token: cleanToken }); 
    } catch (err: any) {
      setError(err.user_message || "Token injection failed.");
      setLoading(false);
    }
  };

  const handleSaveCredentials = async (e: React.FormEvent) => {
    e.preventDefault(); setLoading(true); setError(null);
    try {
      await invoke('save_discord_credentials', { clientId, clientSecret });
      setNeedsCredentials(false);
      setTimeout(() => handleLoginOAuth(), 200);
    } catch (err: any) { setError("Failed to persist config."); setLoading(false); }
  };

  const toggleChannel = (id: string) => {
    const next = new Set(selectedChannels);
    if (next.has(id)) next.delete(id); else next.add(id);
    setSelectedChannels(next);
  };

  const toggleGuildToLeave = (id: string) => {
    const next = new Set(selectedGuildsToLeave);
    if (next.has(id)) next.delete(id); else next.add(id);
    setSelectedGuildsToLeave(next);
  };

  const handleSelectGuild = async (guild: Guild) => {
    setSelectedGuild(guild); setChannels(null); setSelectedChannels(new Set());
    if (appMode === 'messages') {
      setLoading(true);
      try {
        const fetched: Channel[] = await invoke('fetch_channels', { guildId: guild.id });
        setChannels(fetched);
      } catch (err: any) { setError("Channel mapping failed."); } finally { setLoading(false); }
    }
  };

  const startPurge = async () => {
    if (confirmText !== (appMode === 'messages' ? 'DELETE' : 'LEAVE')) return;
    setShowConfirmModal(false);
    setIsProcessing(true);
    setConfirmText('');

    try {
      if (appMode === 'messages') {
        const now = Date.now();
        let startTime = timeRange === '24h' ? now - 86400000 : timeRange === '7d' ? now - 604800000 : undefined;
        await invoke('bulk_delete_messages', { channelIds: Array.from(selectedChannels), startTime, endTime: undefined });
      } else {
        await invoke('bulk_leave_guilds', { guildIds: Array.from(selectedGuildsToLeave) });
      }
    } catch (err: any) {
      setError(err.user_message || "Operation failed.");
      setIsProcessing(false);
    }
  };

  if (needsCredentials) {
    return (
      <div className="min-h-screen bg-black text-white flex items-center justify-center p-6">
        <motion.div initial={{ scale: 0.95, opacity: 0 }} animate={{ scale: 1, opacity: 1 }} className="w-full max-w-md bg-gray-900 border border-gray-800 rounded-3xl p-10 shadow-2xl relative z-50">
          <div className="flex items-center gap-4 mb-10">
            <button onClick={() => setNeedsCredentials(false)} className="p-3 bg-gray-800 rounded-2xl hover:bg-gray-700 transition-all focus:outline-none"><ArrowLeft className="w-5 h-5 text-gray-400" /></button>
            <h2 className="text-3xl font-black uppercase tracking-tighter italic text-blue-500">Engine Config</h2>
          </div>
          <form onSubmit={handleSaveCredentials} className="space-y-8">
            <div className="space-y-3">
              <label className="text-[10px] font-black text-gray-500 uppercase tracking-widest ml-3 flex items-center gap-2">Application ID <HelpMarker content="Found in 'General Information' tab of your Discord Dev App." /></label>
              <input type="text" required value={clientId} onChange={e => setClientId(e.target.value)} className="w-full bg-black border border-gray-800 p-5 rounded-2xl focus:border-blue-500 outline-none font-mono text-xs shadow-inner" placeholder="Application ID" />
            </div>
            <div className="space-y-3">
              <label className="text-[10px] font-black text-gray-500 uppercase tracking-widest ml-3 flex items-center gap-2">Client Secret <HelpMarker content="Found in 'OAuth2' tab of your Discord Dev App." /></label>
              <input type="password" required value={clientSecret} onChange={e => setClientSecret(e.target.value)} className="w-full bg-black border border-gray-800 p-5 rounded-2xl focus:border-blue-500 outline-none font-mono text-xs shadow-inner" placeholder="••••••••" />
            </div>
            <button type="submit" className="w-full bg-blue-600 hover:bg-blue-700 py-6 rounded-2xl font-black text-xs uppercase tracking-widest transition-all shadow-xl shadow-blue-900/20 active:scale-95">Save & Initialize Handshake</button>
            <div className="text-center mt-10 border-t border-gray-800 pt-8">
              <a href="https://discord.com/developers/applications" target="_blank" rel="noopener noreferrer" className="text-[10px] text-blue-500 font-bold uppercase underline underline-offset-8 decoration-2 hover:text-blue-400">Discord Developer Portal</a>
            </div>
          </form>
        </motion.div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-black text-gray-100 font-sans p-10 selection:bg-blue-500/30 overflow-x-hidden flex flex-col">
      <header className="max-w-6xl mx-auto w-full mb-12 flex items-center justify-between border-b border-gray-900 pb-8 relative z-40">
        <div className="space-y-1">
          <h1 className="text-5xl font-black tracking-tighter text-white uppercase italic leading-none">DISCORD PURGE</h1>
          <p className="text-[10px] text-gray-600 font-black tracking-[0.6em] uppercase flex items-center gap-2 leading-none">
            <div className="w-4 h-px bg-blue-600" /> PRIVACY ENFORCEMENT UNIT v5.0
          </p>
        </div>
        
        {isAuthenticated && (
          <div className="flex items-center gap-4">
            <div className="flex bg-gray-900 border border-gray-800 rounded-2xl p-1">
              <button 
                onClick={() => { setAppMode('messages'); setSelectedGuild(null); }}
                className={`px-6 py-2 rounded-xl text-[10px] font-black uppercase tracking-widest transition-all ${appMode === 'messages' ? 'bg-blue-600 text-white shadow-lg' : 'text-gray-500 hover:text-white'}`}
              >
                Messages
              </button>
              <button 
                onClick={() => { setAppMode('servers'); setSelectedGuild(null); }}
                className={`px-6 py-2 rounded-xl text-[10px] font-black uppercase tracking-widest transition-all ${appMode === 'servers' ? 'bg-blue-600 text-white shadow-lg' : 'text-gray-500 hover:text-white'}`}
              >
                Servers
              </button>
            </div>

            <div className="flex items-center gap-6 bg-gray-900 border border-gray-800 p-3 pr-8 rounded-[2rem] shadow-2xl">
              {user?.avatar ? <img src={`https://cdn.discordapp.com/avatars/${user.id}/${user.avatar}.png`} className="w-12 h-12 rounded-2xl border border-white/5 shadow-lg" /> : <div className="w-12 h-12 rounded-2xl bg-blue-600 flex items-center justify-center font-black text-xl shadow-lg">{user?.username[0]}</div>}
              <div>
                <p className="text-xs font-black uppercase tracking-tight leading-none">{user?.username}</p>
                <p className="text-[9px] text-green-500 font-black uppercase tracking-widest mt-1 flex items-center gap-1.5">
                  <div className="w-1.5 h-1.5 bg-green-500 rounded-full animate-pulse shadow-[0_0_8px_green]" /> AUTHORIZED
                </p>
              </div>
              <button onClick={setUnauthenticated} className="p-3 hover:bg-red-500/10 rounded-2xl text-gray-600 hover:text-red-500 transition-all border border-transparent hover:border-red-500/20 ml-2 focus:outline-none"><XCircle className="w-5 h-5" /></button>
            </div>
          </div>
        )}
      </header>

      <main className="max-w-6xl mx-auto w-full flex-1 relative z-30">
        <AnimatePresence mode="wait">
          {!isAuthenticated ? (
            <motion.div initial={{ opacity: 0, y: 10 }} animate={{ opacity: 1, y: 0 }} exit={{ opacity: 0, y: -10 }} className="grid grid-cols-1 md:grid-cols-2 gap-12">
              <section className="space-y-10">
                <div className="bg-gray-900 border border-gray-800 rounded-[3rem] p-10 shadow-2xl flex flex-col min-h-[300px]">
                  <div className="flex items-center justify-between mb-10">
                    <h3 className="text-xs font-black text-gray-500 uppercase tracking-[0.4em] flex items-center gap-3"><Monitor className="w-4 h-4 text-blue-500" /> Environment</h3>
                    <HelpMarker content="Direct connection handshake with your running Discord Desktop client." />
                  </div>
                  <div className="space-y-6 flex-1 flex flex-col justify-center">
                    <div className="flex items-center justify-between p-6 bg-black rounded-[1.8rem] border border-white/5 shadow-inner">
                      <span className="text-xs font-black uppercase tracking-[0.2em] text-gray-400">Desktop Node</span>
                      <span className={`text-[9px] font-black px-4 py-1.5 rounded-full ${discordStatus?.is_running ? 'bg-green-500/10 text-green-500 shadow-[0_0_15px_rgba(34,197,94,0.2)]' : 'bg-gray-800 text-gray-600'}`}>{discordStatus?.is_running ? 'ACTIVE' : 'OFFLINE'}</span>
                    </div>
                    <button onClick={handleLoginRPC} disabled={!discordStatus?.rpc_available} className={`w-full flex items-center justify-between p-6 bg-black rounded-[1.8rem] border transition-all ${discordStatus?.rpc_available ? 'border-blue-500 hover:bg-blue-500/5 hover:scale-[1.02]' : 'border-white/5 opacity-40 cursor-not-allowed'} focus:outline-none`}>
                      <span className="text-sm font-black uppercase tracking-[0.1em] text-blue-400 italic">Instant Link</span>
                      <div className="flex items-center gap-2"><ShieldCheck className={`w-5 h-5 ${discordStatus?.rpc_available ? 'text-blue-500 shadow-[0_0_10px_blue]' : 'text-gray-700'}`} /><span className="text-[10px] font-black uppercase">{discordStatus?.rpc_available ? 'READY' : 'WAITING'}</span></div>
                    </button>
                  </div>
                </div>

                <div className="bg-gray-900 border border-gray-800 rounded-[3rem] p-12 text-center shadow-2xl group hover:border-blue-500/30 transition-all relative overflow-hidden">
                  <div className="absolute inset-0 bg-blue-500/5 opacity-0 group-hover:opacity-100 transition-opacity pointer-events-none" />
                  <Smartphone className="w-14 h-14 text-gray-700 mx-auto mb-8 group-hover:text-blue-500 transition-colors" />
                  <h3 className="text-2xl font-black mb-3 uppercase tracking-tighter italic">QR Handshake</h3>
                  <p className="text-gray-500 text-[10px] mb-10 px-8 font-black uppercase tracking-widest leading-relaxed">Secure mobile-to-desktop session bridge.</p>
                  <button onClick={handleLoginQR} className="w-full bg-blue-600/10 text-blue-400 border border-blue-500/30 py-6 rounded-[2rem] font-black text-[10px] uppercase tracking-[0.3em] hover:bg-blue-600 hover:text-white transition-all shadow-xl hover:shadow-blue-500/30 focus:outline-none">Generate Handshake</button>
                </div>
              </section>

              <section className="space-y-10">
                <div className="bg-gradient-to-br from-blue-600 via-blue-700 to-indigo-900 rounded-[3rem] p-12 shadow-2xl shadow-blue-900/40 relative overflow-hidden group border border-white/5 flex flex-col min-h-[350px]">
                  <Globe className="absolute -right-8 -bottom-8 w-48 h-48 text-white/10 group-hover:rotate-12 transition-transform duration-700 pointer-events-none" />
                  <div className="relative z-10 flex flex-col h-full">
                    <div className="flex items-center justify-between mb-4">
                      <h3 className="text-3xl font-black tracking-tighter italic uppercase leading-none">Official Gate</h3>
                      <HelpMarker content="Authorized OAuth2 protocol. Redirects to official Discord domain." />
                    </div>
                    <p className="text-blue-100/70 text-[11px] mb-auto leading-relaxed font-black uppercase tracking-widest pt-2">Encrypted Authorization Loop.</p>
                    <button onClick={handleLoginOAuth} className="w-full bg-white text-blue-700 py-6 rounded-[2rem] font-black text-xs uppercase tracking-[0.3em] hover:scale-[1.03] active:scale-95 transition-all shadow-2xl mt-10 focus:outline-none">Initialize Flow</button>
                  </div>
                </div>

                <button onClick={() => setAuthMethod('token')} className="w-full bg-gray-900 border border-gray-800 p-10 rounded-[3rem] flex items-center justify-between hover:bg-gray-800 transition-all group shadow-2xl text-left focus:outline-none">
                  <div className="flex items-center gap-8">
                    <div className="bg-red-500/10 p-5 rounded-[1.8rem] text-red-500 group-hover:scale-110 transition-transform shadow-inner border border-red-500/10"><Key className="w-8 h-8" /></div>
                    <div>
                      <h4 className="text-xl font-black tracking-tighter uppercase leading-none italic">Manual Inject</h4>
                      <p className="text-[10px] text-gray-600 font-black uppercase tracking-[0.3em] mt-2 leading-none">Secure Bypass Protocol</p>
                    </div>
                  </div>
                  <HelpMarker content="Direct Injection of User Auth Signature. Warning: Bypasses security handshake." />
                </button>
              </section>
            </motion.div>
          ) : authMethod === 'qr' ? (
            <motion.div initial={{ opacity: 0, scale: 0.95 }} animate={{ opacity: 1, scale: 1 }} className="max-w-md mx-auto text-center bg-gray-900 border border-gray-800 p-14 rounded-[4rem] shadow-2xl relative overflow-hidden">
              <button onClick={() => { setAuthMethod('none'); setQrUrl(null); setQrScanned(false); }} className="flex items-center gap-3 text-[10px] font-black text-gray-600 hover:text-white uppercase tracking-[0.4em] mb-12 transition-all focus:outline-none"><ArrowLeft className="w-4 h-4" /> ABORT HANDSHAKE</button>
              <div className="bg-white p-8 rounded-[3.5rem] inline-block mb-12 shadow-[0_0_60px_rgba(255,255,255,0.1)] border-4 border-blue-500/20">
                {qrUrl ? <QRCodeSVG value={qrUrl} size={220} level="H" includeMargin={true} /> : <div className="w-[220px] h-[220px] flex items-center justify-center bg-gray-50 rounded-[2.5rem]"><div className="w-12 h-12 border-4 border-blue-500 border-t-transparent rounded-full animate-spin" /></div>}
              </div>
              <p className="text-[10px] font-black text-gray-500 leading-relaxed px-8 uppercase tracking-[0.3em] italic">
                {qrScanned ? <span className="text-green-500 flex items-center justify-center gap-3 animate-pulse italic"><CheckCircle2 className="w-6 h-6" /> SIGNATURE RECEIVED. FINALIZING...</span> : "Scan this secure signature with the Discord mobile app scanner (Settings > Scan QR)."}
              </p>
            </motion.div>
          ) : (
            <motion.div initial={{ opacity: 0, scale: 0.95 }} animate={{ opacity: 1, scale: 1 }} className="max-w-2xl mx-auto bg-gray-900 border border-gray-800 p-14 rounded-[4rem] shadow-2xl relative z-50">
              <button onClick={() => setAuthMethod('none')} className="flex items-center gap-3 text-[10px] font-black text-gray-600 hover:text-white uppercase tracking-[0.4em] mb-14 transition-all focus:outline-none"><ArrowLeft className="w-4 h-4" /> ABORT HANDSHAKE</button>
              <div className="flex items-center justify-between mb-10 border-b border-gray-800 pb-10">
                <h3 className="text-3xl font-black tracking-tighter uppercase italic leading-none">Token Injection</h3>
                <HelpMarker content={<div className="space-y-4 font-black uppercase text-[10px] tracking-widest"><p className="text-red-500 underline decoration-2">Protocol Warning</p><p>Injection grants TOTAL account access. Protect payload signature.</p><hr className="border-gray-800" /><p className="text-gray-500">Extraction Logic:</p><ol className="list-decimal list-inside space-y-3 font-mono text-[9px]"><li>Open Discord Web</li><li>Tap F12</li><li>Select Network</li><li>Filter /api</li><li>Capture Authorization Header</li></ol></div>} />
              </div>
              <form onSubmit={handleLoginToken} className="space-y-10">
                <div className="space-y-4">
                  <p className="text-[10px] text-gray-600 uppercase font-black tracking-[0.5em] ml-4 italic">Injection Payload Signature</p>
                  <input type="password" value={manualToken} onChange={e => setManualToken(e.target.value)} className="w-full bg-black border border-gray-800 p-6 rounded-[2rem] focus:border-red-500 outline-none font-mono text-xs shadow-inner transition-all text-red-500" placeholder="NULL_SIGNATURE" />
                </div>
                <button type="submit" disabled={!manualToken} className="w-full bg-red-600 hover:bg-red-700 py-7 rounded-[2.2rem] font-black text-xs uppercase tracking-[0.5em] transition-all disabled:opacity-50 shadow-2xl shadow-red-900/20 active:scale-95">Establish Link</button>
              </form>
            </motion.div>
          )}
        </AnimatePresence>

        {isAuthenticated && appMode === 'messages' && (
          <motion.div initial={{ opacity: 0 }} animate={{ opacity: 1 }} className="grid grid-cols-1 lg:grid-cols-12 gap-12 items-start mt-10">
            <aside className="lg:col-span-4 space-y-8">
              <h3 className="text-[10px] font-black text-gray-600 uppercase tracking-[0.5em] ml-6 flex items-center gap-3 font-mono"><Server className="w-4 h-4 text-blue-500 shadow-[0_0_10px_blue]" /> Data sources</h3>
              <div className="bg-gray-900/40 border border-gray-800 rounded-[3.5rem] overflow-hidden max-h-[600px] overflow-y-auto custom-scrollbar p-4 space-y-3 shadow-inner">
                {guilds?.map(g => (
                  <button key={g.id} onClick={() => handleSelectGuild(g)} className={`w-full flex items-center gap-6 p-6 rounded-[2.5rem] transition-all relative overflow-hidden group ${selectedGuild?.id === g.id ? 'bg-blue-600 text-white shadow-2xl' : 'hover:bg-white/5 text-gray-400'} focus:outline-none`}>
                    {g.icon ? <img src={`https://cdn.discordapp.com/icons/${g.id}/${g.icon}.png`} className="w-14 h-14 rounded-2xl shadow-xl transition-transform group-hover:scale-110" /> : <div className="w-14 h-14 rounded-2xl bg-gray-800 flex items-center justify-center font-black text-xl shadow-lg border border-white/5">{g.name[0]}</div>}
                    <span className="text-sm font-black tracking-tighter truncate uppercase italic leading-none">{g.name}</span>
                    {selectedGuild?.id === g.id && <div className="absolute right-8 w-2 h-2 bg-white rounded-full shadow-[0_0_15px_white]" />}
                  </button>
                ))}
              </div>
            </aside>

            <div className="lg:col-span-8">
              {selectedGuild ? (
                <motion.div initial={{ opacity: 0, x: 20 }} animate={{ opacity: 1, x: 0 }} className="bg-gray-900 border border-gray-800 p-12 rounded-[4.5rem] space-y-12 shadow-2xl relative overflow-hidden backdrop-blur-3xl flex flex-col min-h-[650px]">
                  <div className="absolute top-0 right-0 p-14 opacity-[0.03] pointer-events-none"><Trash2 className="w-48 h-48 rotate-12" /></div>
                  <div className="relative z-10 border-b border-gray-800 pb-12 flex flex-col items-center text-center">
                    <h3 className="text-5xl font-black text-white tracking-tighter italic uppercase leading-none">{selectedGuild.name}</h3>
                    <div className="flex items-center gap-4 mt-6 bg-blue-600/5 px-6 py-2 rounded-full border border-blue-500/10 shadow-inner">
                      <div className="w-2.5 h-2.5 bg-blue-500 rounded-full animate-pulse shadow-[0_0_15px_blue]" />
                      <p className="text-[10px] text-blue-500 font-black uppercase tracking-[0.6em] italic leading-none">Node Calibrating</p>
                    </div>
                  </div>
                  
                  <div className="space-y-12 flex-1 relative z-10">
                    <div className="space-y-6">
                      <label className="text-[10px] font-black text-gray-600 uppercase tracking-[0.5em] ml-8 flex items-center gap-3 font-mono"><Clock className="w-4 h-4 text-gray-500" /> Temporal range</label>
                      <div className="flex gap-4 p-3 bg-black/40 rounded-[2.2rem] border border-gray-800 shadow-inner">
                        {(['24h', '7d', 'all'] as const).map(r => (
                          <button key={r} onClick={() => setTimeRange(r)} className={`flex-1 py-5 rounded-[1.8rem] text-[10px] font-black uppercase tracking-widest transition-all ${timeRange === r ? 'bg-white text-black shadow-[0_0_30px_rgba(255,255,255,0.2)] scale-105' : 'text-gray-600 hover:text-gray-300'} focus:outline-none`}>{r === '24h' ? '24 HOURS' : r === '7d' ? '7 DAYS' : 'FULL DEPTH'}</button>
                        ))}
                      </div>
                    </div>

                    <div className="space-y-6 pt-10">
                      <div className="flex items-center justify-between px-8">
                        <label className="text-[10px] font-black text-gray-600 uppercase tracking-[0.5em] flex items-center gap-3 font-mono"><Hash className="w-4 h-4 text-gray-500" /> Node buffers</label>
                        <button onClick={() => setSelectedChannels(new Set(channels?.map(c => c.id)))} className="text-[10px] font-black text-blue-500 hover:text-blue-400 uppercase tracking-[0.4em] underline decoration-2 underline-offset-8 transition-all focus:outline-none">Map all nodes</button>
                      </div>
                      <div className="grid grid-cols-1 md:grid-cols-2 gap-5 max-h-[400px] overflow-y-auto pr-4 custom-scrollbar p-2">
                        {channels?.map(c => (
                          <button key={c.id} onClick={() => toggleChannel(c.id)} className={`flex items-center justify-between p-6 rounded-[3rem] border-2 transition-all text-left ${selectedChannels.has(c.id) ? 'bg-blue-600/10 border-blue-500 shadow-[0_0_30px_rgba(59,130,246,0.15)] text-blue-100' : 'bg-black/30 border-gray-800 text-gray-600 hover:border-gray-700'} focus:outline-none`}>
                            <span className="truncate font-black text-[11px] tracking-tight uppercase italic leading-none">#{c.name}</span>
                            <div className={`w-6 h-6 rounded-xl border-2 flex items-center justify-center transition-all ${selectedChannels.has(c.id) ? 'bg-blue-500 border-blue-500 scale-110 shadow-lg' : 'border-gray-800'}`}>{selectedChannels.has(c.id) && <CheckCircle2 className="w-4 h-4 text-white" />}</div>
                          </button>
                        ))}
                      </div>
                    </div>
                  </div>

                  <button disabled={selectedChannels.size === 0 || isProcessing} onClick={() => setShowConfirmModal(true)} className="w-full bg-red-600 hover:bg-red-700 py-8 rounded-[3rem] font-black text-sm uppercase tracking-[0.5em] shadow-[0_0_50px_rgba(220,38,38,0.25)] transition-all hover:scale-[1.01] active:scale-95 disabled:opacity-50 relative z-10 focus:outline-none mt-10">Execute Purge Sequence ({selectedChannels.size})</button>
                </motion.div>
              ) : (
                <div className="h-full min-h-[650px] flex flex-col items-center justify-center border-2 border-dashed border-gray-800 rounded-[5rem] p-24 text-center opacity-20 shadow-inner">
                  <ShieldCheck className="w-24 h-24 mb-10 text-blue-500 drop-shadow-[0_0_20px_blue]" />
                  <p className="font-black uppercase tracking-[0.8em] text-[11px] font-mono leading-relaxed text-blue-500">Awaiting Calibration <br/> Map Target Buffer to Begin</p>
                </div>
              )}
            </div>
          </motion.div>
        )}

        {isAuthenticated && appMode === 'servers' && (
          <motion.div initial={{ opacity: 0 }} animate={{ opacity: 1 }} className="mt-10">
            <div className="bg-gray-900 border border-gray-800 rounded-[4rem] p-12 shadow-2xl relative overflow-hidden backdrop-blur-3xl">
              <div className="flex items-center justify-between mb-12 border-b border-gray-800 pb-8 px-4">
                <div>
                  <h3 className="text-4xl font-black text-white tracking-tighter uppercase italic leading-none">Server Management</h3>
                  <p className="text-[10px] text-blue-500 font-black uppercase tracking-[0.5em] mt-3">Bulk Departure Protocol</p>
                </div>
                <button 
                  onClick={() => setSelectedGuildsToLeave(new Set(guilds?.map(g => g.id)))}
                  className="text-[10px] font-black text-blue-500 hover:text-blue-400 uppercase tracking-widest underline decoration-2 underline-offset-8 transition-all focus:outline-none"
                >
                  Select All
                </button>
              </div>

              <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6 max-h-[600px] overflow-y-auto custom-scrollbar p-4">
                {guilds?.map(g => (
                  <button
                    key={g.id}
                    onClick={() => toggleGuildToLeave(g.id)}
                    className={`flex items-center gap-5 p-6 rounded-[2.5rem] border-2 transition-all text-left ${selectedGuildsToLeave.has(g.id) ? 'bg-red-600/10 border-red-500 shadow-[0_0_30px_rgba(220,38,38,0.15)] text-red-100' : 'bg-black/30 border-gray-800 text-gray-500 hover:border-gray-700'} focus:outline-none`}
                  >
                    {g.icon ? <img src={`https://cdn.discordapp.com/icons/${g.id}/${g.icon}.png`} className="w-12 h-12 rounded-2xl shadow-xl transition-transform" /> : <div className="w-12 h-12 rounded-2xl bg-gray-800 flex items-center justify-center font-black text-xl shadow-lg">{g.name[0]}</div>}
                    <div className="flex-1 min-w-0">
                      <p className="text-sm font-black tracking-tighter truncate uppercase italic leading-none">{g.name}</p>
                    </div>
                    <div className={`w-6 h-6 rounded-xl border-2 flex items-center justify-center transition-all ${selectedGuildsToLeave.has(g.id) ? 'bg-red-500 border-red-500 scale-110 shadow-lg' : 'border-gray-800'}`}>{selectedGuildsToLeave.has(g.id) && <CheckCircle2 className="w-4 h-4 text-white" />}</div>
                  </button>
                ))}
              </div>

              <div className="mt-12 flex justify-center border-t border-gray-800 pt-10 px-4">
                <button 
                  disabled={selectedGuildsToLeave.size === 0 || isProcessing}
                  onClick={() => setShowConfirmModal(true)}
                  className="w-full max-w-2xl bg-red-600 hover:bg-red-700 py-8 rounded-[3rem] font-black text-sm uppercase tracking-[0.5em] shadow-[0_0_50px_rgba(220,38,38,0.25)] transition-all hover:scale-[1.01] active:scale-95 disabled:opacity-50 focus:outline-none"
                >
                  Terminate {selectedGuildsToLeave.size} Server Connections
                </button>
              </div>
            </div>
          </motion.div>
        )}
      </main>

      {/* Confirmation Modal */}
      <AnimatePresence>
        {showConfirmModal && (
          <div className="fixed inset-0 bg-black/98 backdrop-blur-[100px] flex items-center justify-center p-10 z-[300]">
            <motion.div initial={{ scale: 0.9, y: 30 }} animate={{ scale: 1, y: 0 }} className="bg-gray-900 border border-red-500/40 rounded-[5rem] p-20 max-w-2xl w-full space-y-12 text-center shadow-[0_0_200px_rgba(220,38,38,0.2)] relative overflow-hidden">
              <ShieldAlert className="w-24 h-24 text-red-600 mx-auto drop-shadow-[0_0_40px_red] mb-4" />
              <h2 className="text-6xl font-black tracking-tighter uppercase italic leading-none">Authorization</h2>
              <p className="text-gray-500 font-black text-xs uppercase tracking-[0.3em] leading-relaxed px-14">
                {appMode === 'messages' ? (
                  <>You are authorizing a permanent destructive sequence for <span className="text-white font-black underline decoration-red-500 decoration-2 underline-offset-8 tracking-[0.1em]">{selectedChannels.size} target buffers</span>. <br/><br/> Action status: PERMANENT.</>
                ) : (
                  <>You are authorizing the permanent termination of <span className="text-white font-black underline decoration-red-500 decoration-2 underline-offset-8 tracking-[0.1em]">{selectedGuildsToLeave.size} server connections</span>. <br/><br/> This cannot be reversed.</>
                )}
              </p>
              <div className="space-y-5">
                <label className="text-[10px] font-black text-gray-600 uppercase tracking-[0.6em] font-mono italic text-center block">Security Signature Required (<span className="text-red-500 underline decoration-2">{appMode === 'messages' ? 'DELETE' : 'LEAVE'}</span>)</label>
                <input type="text" value={confirmText} onChange={e => setConfirmText(e.target.value.toUpperCase())} className="w-full bg-black/80 border border-gray-800 p-8 rounded-[2.8rem] text-center text-red-500 font-black tracking-[1.2em] outline-none text-3xl shadow-inner focus:border-red-500/50 transition-all uppercase italic" placeholder="SIGNATURE" />
              </div>
              <div className="flex gap-10">
                <button onClick={() => setShowConfirmModal(false)} className="flex-1 py-8 text-gray-600 font-black uppercase text-[11px] tracking-[0.6em] border border-gray-800 rounded-[2.5rem] hover:bg-white/5 transition-all focus:outline-none">Abort Protocol</button>
                <button 
                  disabled={confirmText !== (appMode === 'messages' ? 'DELETE' : 'LEAVE')} 
                  onClick={startPurge} 
                  className={`flex-1 py-8 rounded-[2.5rem] font-black text-[11px] tracking-[0.6em] uppercase transition-all shadow-2xl ${confirmText === (appMode === 'messages' ? 'DELETE' : 'LEAVE') ? 'bg-red-600 hover:bg-red-700 text-white shadow-red-900/60 scale-105' : 'bg-gray-800 text-gray-700 cursor-not-allowed'} focus:outline-none`}
                >
                  Execute
                </button>
              </div>
            </motion.div>
          </div>
        )}
      </AnimatePresence>

      <AnimatePresence>
        {isProcessing && (
          <div className="fixed inset-0 bg-black flex flex-col items-center justify-center p-10 text-center z-[400] overflow-hidden">
            <motion.h2 animate={{ scale: [1, 1.02, 1], opacity: [0.6, 1, 0.6] }} transition={{ repeat: Infinity, duration: 1.5 }} className="text-9xl font-black italic tracking-tighter mb-32 uppercase leading-none italic shadow-[0_0_50px_rgba(255,255,255,0.1)]">
              {appMode === 'messages' ? 'Purging' : 'Terminating'}
            </motion.h2>
            
            {(deletionProgress || leaveProgress) ? (
              <div className="w-full max-w-2xl space-y-20 relative">
                <div className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-[700px] h-[700px] bg-blue-600/10 rounded-full blur-[250px] -z-10" />
                <div className="space-y-10 px-10">
                  <div className="flex justify-between text-[11px] font-black text-gray-600 uppercase tracking-[1em] font-mono leading-none px-6">
                    <span>Saturation</span>
                    <span className="text-blue-500 italic shadow-[0_0_10px_blue]">
                      {appMode === 'messages' ? `${deletionProgress?.current_channel} / ${deletionProgress?.total_channels}` : `${leaveProgress?.current} / ${leaveProgress?.total}`}
                    </span>
                  </div>
                  <div className="w-full h-5 bg-gray-950 rounded-full overflow-hidden border border-white/5 p-1 shadow-[0_0_30px_rgba(0,0,0,0.5)]">
                    <motion.div 
                      animate={{ width: `${appMode === 'messages' ? ((deletionProgress?.current_channel || 0) / (deletionProgress?.total_channels || 1)) * 100 : ((leaveProgress?.current || 0) / (leaveProgress?.total || 1)) * 100}%` }} 
                      className="h-full bg-gradient-to-r from-blue-600 via-purple-600 to-red-600 shadow-[0_0_50px_blue]" 
                    />
                  </div>
                </div>
                <div className="grid grid-cols-2 gap-16 px-6">
                  <div className="bg-gray-900/50 p-14 rounded-[4rem] border border-gray-800 shadow-2xl backdrop-blur-3xl relative overflow-hidden group">
                    <div className="absolute top-0 left-0 w-full h-1 bg-blue-500/40 group-hover:h-2 transition-all" />
                    <p className="text-[11px] text-gray-600 font-black uppercase tracking-[0.8em] mb-6 text-left italic font-mono">Phase</p>
                    <p className="text-5xl font-black text-blue-500 uppercase tracking-tighter italic text-left leading-none shadow-[0_0_15px_rgba(59,130,246,0.3)]">
                      {appMode === 'messages' ? deletionProgress?.status : leaveProgress?.status}
                    </p>
                  </div>
                  <div className="bg-gray-900/50 p-14 rounded-[4rem] border border-gray-800 shadow-2xl backdrop-blur-3xl relative overflow-hidden group">
                    <div className="absolute top-0 left-0 w-full h-1 bg-red-500/40 group-hover:h-2 transition-all" />
                    <p className="text-[11px] text-gray-600 font-black uppercase tracking-[0.8em] mb-6 text-left italic font-mono">
                      {appMode === 'messages' ? 'Nullified' : 'Severed'}
                    </p>
                    <p className="text-5xl font-black text-red-500 tracking-tighter italic text-left leading-none shadow-[0_0_15px_rgba(220,38,38,0.3)]">
                      {appMode === 'messages' ? deletionProgress?.deleted_count : leaveProgress?.current}
                    </p>
                  </div>
                </div>
                <div className="p-10 bg-white/5 rounded-[3.5rem] border border-white/5 inline-block backdrop-blur-3xl shadow-inner group transition-all hover:bg-white/10">
                  <p className="text-[11px] text-gray-500 font-black uppercase tracking-[1em] italic font-mono uppercase">
                    Active stream: <span className="text-white group-hover:text-blue-400 transition-colors uppercase italic">
                      {appMode === 'messages' ? `#${channels?.find(c => c.id === deletionProgress?.channel_id)?.name || '0xUNKNOWN'}` : guilds?.find(g => g.id === leaveProgress?.guild_id)?.name || '0xUNKNOWN'}
                    </span>
                  </p>
                </div>
              </div>
            ) : <div className="w-40 h-40 border-8 border-blue-500/10 border-t-blue-500 rounded-[4rem] animate-spin shadow-[0_0_100px_blue]" />}
          </div>
        )}
      </AnimatePresence>
    </div>
  );
}

export default App;
