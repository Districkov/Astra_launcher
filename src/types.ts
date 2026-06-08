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
  type: "info" | "success" | "error" | "warning" | "update";
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

// ── Tauri Command Response Types ──

export interface ServerStatusResponse {
  online: boolean;
  players: number;
  max_players: number;
  info: Record<string, unknown> | null;
}

export interface ServerPingResponse {
  ping: number;
  error?: string;
}

export interface PrecacheCheckResponse {
  needed: boolean;
  reason: string;
  cache_files?: number;
  cache_size_mb?: number;
}

export interface PrecacheResultResponse {
  success: boolean;
  extracted_files: number;
  cache_dir: string;
}

export interface DownloadProgressPayload {
  downloaded: number;
  total: number;
  percent: number;
}

// ── Store / Магазин ──

export type StoreCategory = "donate" | "transport" | "property";

export interface StoreItem {
  id: string;
  name: string;
  description: string;
  price: number;           // в рублях
  oldPrice?: number;       // старая цена (для скидки)
  category: StoreCategory;
  image?: string;          // URL или путь к картинке
  badge?: string;          // "Хит", "Скидка", "Новинка" и т.д.
  popular?: boolean;       // популярный товар
  paymentUrl?: string;     // ссылка на платёжный шлюз
}

export interface StoreCart {
  items: StoreItem[];
  totalPrice: number;
}

export interface PrecacheProgressPayload {
  phase: "download" | "extract";
  downloaded?: number;
  total?: number;
  percent: number;
  extracted?: number;
  total_files?: number;
}

export interface LauncherConfigResponse {
  server_address: string;
  fivem_download_url: string;
  precache_url: string;
}

export interface GalleryUploadResponse {
  id: string;
  url: string;
  thumb: string;
  author: string;
  timestamp: number;
}

export type UpdateEventPayload = {
  event: "Started" | "Progress" | "Finished";
  data: {
    contentLength?: number;
    chunkLength?: number;
  };
};
