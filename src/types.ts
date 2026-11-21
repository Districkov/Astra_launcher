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
  rageDir: string;
  clientReady: boolean;
  username: string;
  updateAvailable: boolean;
  updateChecked: boolean;
  updateInfo: UpdateInfo;
}

// ── Tauri Command Response Types ──

export interface ServerStatusResponse {
  online: boolean;
  players: number;
  max_players: number;
  info: Record<string, unknown> | null;
  /** How reachability was established: "http" | "tcp" | "none". */
  reachability?: "http" | "tcp" | "none";
  /** Whether the server answered the stats endpoints. */
  has_stats?: boolean;
  ping?: number;
}

export interface ServerPingResponse {
  ping: number;
  error?: string;
}

export interface FileEntry {
  path: string;
  sha256: string;
  size: number;
}

export interface CheckResult {
  valid: boolean;
  missing: FileEntry[];
  outdated: FileEntry[];
  total_download_size: number;
}

export interface DownloadProgressPayload {
  downloaded: number;
  total: number;
  percent: number;
  current_file?: string;
}

export interface CacheCheckResponse {
  needed: boolean;
  reason: string;
  version_local: string;
  version_server: string;
}

export interface CacheDownloadProgress {
  phase: "download" | "extract";
  downloaded?: number;
  total?: number;
  percent: number;
  extracted?: number;
  total_files?: number;
}

export interface Patch {
  name: string;
  target?: string;
  offset: string;
  original: string;
  replacement: string;
}

export interface PatchManifest {
  version: string;
  gta_build: string;
  ragemp_version: string;
  patches: Patch[];
}

/** Patch resolution response from `get_patch_manifest`. */
export interface PatchManifestResponse extends PatchManifest {
  count: number;
  /** `builtin` = the table ported from tiny-ragemp-launcher, `server` = remote. */
  source: "builtin" | "server";
  upstream: string;
}

/** Patch state of the installed updater.exe. */
export interface PatchStatusResponse {
  applied: number;
  total: number;
  patched: boolean;
  /** Whether updater.exe still matches the verified reference build. */
  reference_build: boolean;
  manifest_version: string;
  updater_size: number;
}

/** Connect method for the direct join. */
export type ConnectMode = "raw" | "hosts";

/**
 * Which binary the launcher hands control to.
 *
 * - `updater`  — official path: patch + run `updater.exe` (it does the CDN
 *                check, which our patches bypass, then starts the client).
 * - `ragemp_v` — direct path: run `ragemp_v.exe` immediately, skipping both
 *                the updater and the CDN check.
 */
export type LaunchTarget = "updater" | "ragemp_v";

/** Registry values that drive direct connect. */
export interface RegistryState {
  launch_ip: string | null;
  launch_port: string | null;
  launch_type: string | null;
  launch2_ip: string | null;
  launch2_port: string | null;
}

/** Full picture of what the next launch will do. */
export interface LaunchDiagnostic {
  rage_dir: string;
  target: LaunchTarget;
  target_binary_present: boolean;
  updater_present: boolean;
  client_present: boolean;
  server_ip: string;
  server_port: string;
  connect_mode: ConnectMode;
  rui_auto_connect: boolean;
  registry: RegistryState;
  registry_ok: boolean;
  registry_matches_server: boolean;
  rui_injected: boolean;
  /** True when first-launch setup will be performed by the next Play action. */
  rui_pending?: boolean;
  raw_ip_covered: boolean;
  notes: string[];
}

export interface SetLaunchTargetResponse {
  target: LaunchTarget;
  binary_present: boolean | null;
}

export interface WhitelistStatusResponse {
  registry_ip: string | null;
  registry_port: string | null;
  hosts_path: string;
  hosts_writable: boolean;
  hosts_domain: string;
  hosts_domain_whitelisted: boolean;
  hosts_mapped: boolean;
  hosts_mapped_ip: string | null;
  connect_mode: ConnectMode;
  rui_path: string | null;
}

export interface LaunchResult {
  success: boolean;
  pid: number;
  server: string;
  connect_host?: string;
  connect_mode?: ConnectMode;
  patches_applied?: number;
  rui_auto_connect?: boolean;
}

// ── RUI archive editor (rui\index.rui) ──

/** State of the installed `index.rui` archive. */
export interface RuiStatusResponse {
  available: boolean;
  reason?: string;
  path?: string;
  size?: number;
  parsable?: boolean;
  parse_error?: string | null;
  entries?: number;
  contiguous?: boolean;
  app_js_index?: number | null;
  index_html_index?: number | null;
  injected?: boolean;
  backup_exists?: boolean;
  backup_path?: string;
}

export interface RuiAutoConnect {
  enabled: boolean;
  ip: string;
  port: number;
  name: string;
  delay_ms: number;
}

export interface RuiServerEntry {
  name: string;
  ip: string;
  port: number;
  players?: number;
  maxplayers?: number;
  ping?: number;
  lang?: string;
  gamemode?: string;
  url?: string;
}

export interface RuiServerList {
  override: boolean;
  servers: RuiServerEntry[];
}

/** Editor configuration — field names mirror upstream `tools/config.json`. */
export interface RuiConfig {
  title: string | null;
  auto_connect: RuiAutoConnect;
  server_list: RuiServerList;
  /** Theme colour remap, e.g. `{"#f64a46": "#00ff00"}`. */
  colors: Record<string, string>;
  /** UI string replacements as `[[old, new], ...]`. */
  texts: [string, string][];
  /** Image overrides: archive index (as a string) → absolute file path. */
  images: Record<string, string>;
}

/** Result of applying an RUI config. */
export interface RuiEditReport {
  entries_changed: number;
  auto_connect: boolean;
  server_list_entries: number;
  colors_applied: number;
  texts_applied: number;
  title_applied: boolean;
  images_applied: number;
  bytes_before: number;
  bytes_after: number;
  contiguity_ok: boolean;
  warnings: string[];
}

// ── Store / Магазин ──

export type StoreCategory = "donate" | "transport" | "property";

export interface StoreItem {
  id: string;
  name: string;
  description: string;
  price: number;
  oldPrice?: number;
  category: StoreCategory;
  image?: string;
  badge?: string;
  popular?: boolean;
  paymentUrl?: string;
}

export interface StoreCart {
  items: StoreItem[];
  totalPrice: number;
}

export interface LauncherConfigResponse {
  server_address: string;
  client_manifest_url: string;
  cache_info_url: string;
  patches_url: string;
  file_download_base_url: string;
  ragemp_download_url: string;
  rage_dir: string;
  gta5_path: string;
  connect_mode: ConnectMode;
  hosts_domain: string;
  launch_target: LaunchTarget;
}

/**
 * Download progress events emitted by the Tauri updater plugin.
 *
 * Modelled as a discriminated union (matching the plugin's own
 * `DownloadEvent`), so `data` is only present on the events that carry it.
 */
export type UpdateEventPayload =
  | { event: "Started"; data: { contentLength?: number } }
  | { event: "Progress"; data: { chunkLength?: number } }
  | { event: "Finished" };
