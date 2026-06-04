/**
 * ASTRA Launcher — Type Definitions
 * Centralized TypeScript types for the application
 */

export interface MenuItem {
  id: string;
  label: string;
  icon: string;
  group?: string;
}

export interface ServerInfo {
  players: number;
  maxPlayers: number;
  ping: number;
  online: boolean;
}

export interface UpdateInfo {
  current_version: string;
  latest_version: string;
  release_notes: string;
}

export interface Notification {
  id: string;
  message: string;
  type: "info" | "success" | "error" | "warning";
  timestamp: number;
  duration?: number;
}

export interface LaunchPhase {
  label: string;
  icon: string;
}

export interface AppState {
  activeMenu: string;
  isLaunching: boolean;
  statusMessage: string;
  launchPhase: number;
  launchProgress: number;
  serverOnline: boolean;
  serverLoading: boolean;
  serverPlayers: number;
  serverMaxPlayers: number;
  serverPing: number;
  fivemPath: string;
  fivemFound: boolean;
  username: string;
  updateAvailable: boolean;
  updateChecked: boolean;
  updateInfo: UpdateInfo;
}

export interface App {
  activeMenu: string;
  serverOnline: boolean;
  serverLoading: boolean;
  serverPlayers: number;
  serverPing: number;
  username: string;
  menuItems: MenuItem[];
}
