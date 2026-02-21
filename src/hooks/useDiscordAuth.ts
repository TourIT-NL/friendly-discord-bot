import { useState, useCallback, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useAuthStore } from '../store/authStore';
import { DiscordStatus, DiscordIdentity, DiscordUser } from '../types/discord';

export const useDiscordAuth = () => {
  const { 
    setAuthenticated, setLoading, setError, reset, isLoading, view, setView 
  } = useAuthStore();

  const [identities, setIdentities] = useState<DiscordIdentity[]>([]);
  const [discordStatus, setDiscordStatus] = useState<DiscordStatus | null>(null);
  const [qrUrl, setQrUrl] = useState<string | null>(null);
  const [qrScanned, setQrScanned] = useState(false);
  const [clientId, setClientId] = useState('');
  const [clientSecret, setClientSecret] = useState('');
  const [manualToken, setManualToken] = useState('');

  const formatApiError = (err: any, fallback: string) => {
    const msg = typeof err === 'string' ? err : (err.user_message || fallback);
    const detail = err.technical_details ? ` (${err.technical_details})` : "";
    return `${msg}${detail}`;
  };

  const handleApiError = useCallback((err: any, fallback: string) => {
    const formattedMsg = formatApiError(err, fallback);
    console.group(`[API Error] ${fallback}`);
    console.error("Original Error:", err);
    if (err.technical_details) {
      try {
        console.error("Technical Details:", JSON.parse(err.technical_details));
      } catch {
        console.error("Technical Details:", err.technical_details);
      }
    }
    console.groupEnd();
    
    setError(formattedMsg, err);
    setLoading(false);
  }, [setError, setLoading]);

  const checkStatus = useCallback(async () => { 
    try { 
      const status = await invoke<DiscordStatus>('check_discord_status');
      setDiscordStatus(status);
      useAuthStore.getState().resetRetry();
    } catch (err) { 
      const state = useAuthStore.getState();
      if (state.retryCount < 5) {
        state.incrementRetry();
        const backoff = Math.min(1000 * Math.pow(2, state.retryCount), 10000);
        console.warn(`[Status] Check failed. Retrying in ${backoff}ms... (${state.retryCount}/5)`);
        setTimeout(checkStatus, backoff);
      } else {
        console.error("[Status] Maximum retries reached. Discord detection offline.");
        handleApiError(err, "Discord link status unavailable.");
      }
    } 
  }, [handleApiError]);

  const fetchIdentities = useCallback(async () => {
    try { 
      setIdentities(await invoke('list_identities')); 
    } catch (err) { 
      console.error("Failed to fetch identities:", err); 
    }
  }, []);

  const handleLogout = async () => { 
    reset(); 
    setView('manual'); 
  };

  const handleLoginOAuth = async () => { 
    setLoading(true); 
    setError(null); 
    try { 
      await invoke('start_oauth_flow'); 
    } catch (err: any) { 
      if (err.error_code === 'credentials_missing') { 
        setView('setup'); 
        setError(err.user_message); 
      } else { 
        handleApiError(err, "OAuth handshake failed."); 
      } 
      setLoading(false); 
    } 
  };

  const handleLoginQR = async () => { 
    setView('qr'); 
    setQrUrl(null); 
    setQrScanned(false); 
    try { 
      await invoke('start_qr_login_flow'); 
    } catch (err: any) { 
      handleApiError(err, "QR Gateway failed."); 
      setView('auth'); 
    } 
  };

  const handleCancelQR = async () => {
    setLoading(false);
    setView('auth');
    try {
      await invoke('cancel_qr_login');
    } catch (err) {
      console.error("Failed to cancel QR login:", err);
    }
  };

  const handleLoginRPC = async () => { 
    setLoading(true); 
    setError(null); 
    try { 
      await invoke('login_with_rpc'); 
    } catch (err: any) { 
      if (err.error_code === 'credentials_missing') { 
        setView('setup'); 
        setError(err.user_message); 
      } else { 
        handleApiError(err, "RPC handshake failed."); 
      } 
    } 
  };

  const handleLoginToken = async (e: React.FormEvent) => { 
    e.preventDefault(); 
    setLoading(true); 
    try { 
      await invoke('login_with_user_token', { 
        token: manualToken.trim().replace(/^Bearer\s+/i, '').replace(/^"|"$/g, '') 
      }); 
    } catch (err: any) { 
      handleApiError(err, "Identity validation failed."); 
    } 
  };

  const handleSaveConfig = async (e: React.FormEvent) => { 
    e.preventDefault(); 
    setLoading(true); 
    try { 
      await invoke('save_discord_credentials', { clientId, clientSecret }); 
      setView('auth'); 
      setError(null); 
      setTimeout(handleLoginOAuth, 1500); 
    } catch (err: any) { 
      handleApiError(err, "Secure storage failure."); 
    } 
  };
  
  const handleSwitchIdentity = async (id: string) => {
    setLoading(true); 
    try { 
      await invoke('switch_identity', { id }); 
    } catch (err: any) { 
      handleApiError(err, "Switch failed."); 
    } finally { 
      setLoading(false); 
    }
  };

  return {
    view, setView,
    identities, setIdentities,
    discordStatus, setDiscordStatus,
    qrUrl, setQrUrl,
    qrScanned, setQrScanned,
    clientId, setClientId,
    clientSecret, setClientSecret,
    manualToken, setManualToken,
    checkStatus,
    fetchIdentities,
    handleLogout,
    handleLoginOAuth,
    handleLoginQR,
    handleCancelQR,
    handleLoginRPC,
    handleLoginToken,
    handleSaveConfig,
    handleSwitchIdentity,
    handleApiError
  };
};
