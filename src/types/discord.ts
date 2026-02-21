export interface DiscordUser {
  id: string;
  username: string;
  avatar?: string;
  email?: string;
}

export interface DiscordStatus {
  is_running: boolean;
  rpc_available: boolean;
  browser_detected: boolean;
}

export interface Guild {
  id: string;
  name: string;
  icon?: string;
}

export interface Channel {
  id: string;
  name: string;
  channel_type: number;
}

export interface Relationship {
  id: string;
  nickname?: string;
  user: {
    id: string;
    username: string;
    avatar?: string;
  };
  rel_type: number;
}

export interface DiscordIdentity {
  id: string;
  username: string;
  is_oauth: boolean;
}

export interface Progress {
  current: number;
  total: number;
  id: string;
  deleted_count?: number;
  status: string;
}

export interface OperationStatus {
  is_running: boolean;
  is_paused: boolean;
  should_abort: boolean;
}
