/**
 * gameStore.svelte.ts — RAGE:MP client management
 * Svelte 5 runes-based store
 */
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { listen } from "@tauri-apps/api/event";
import { handleError } from "../errorHandling";
import { fmtSize, TIMING } from "../constants";
import type {
  LaunchPhase,
  DownloadProgressPayload,
  PatchManifestResponse,
  PatchStatusResponse,
  WhitelistStatusResponse,
  ConnectMode,
  RuiStatusResponse,
  RuiConfig,
  RuiEditReport,
  LauncherConfigResponse,
  LaunchTarget,
  LaunchDiagnostic,
} from "../types";

// Lazy getter to avoid circular dependency
function getServerStore() {
  return import("./serverStore.svelte").then(m => m.getServerStore());
}

// ── Client path ──
let rageDir = $state("");
let clientReady = $state(false);

// ── File check ──
let isCheckingFiles = $state(false);
let checkResult = $state<unknown>(null);

// ── Client download ──
let isDownloadingClient = $state(false);
let clientDownloadPercent = $state(0);
let clientDownloadFile = $state("");
let clientDownloadError = $state("");

// ── RAGE:MP download ──
let isDownloadingRagemp = $state(false);
let ragempDownloadPercent = $state(0);
let ragempDownloadPhase = $state<"download" | "extract">("download");
let ragempDownloadError = $state("");

// ── Server cache ──
let isCheckingCache = $state(false);
let isDownloadingCache = $state(false);
let cachePercent = $state(0);
let cachePhase = $state<"download" | "extract">("download");
let cacheError = $state("");

// ── Patches ──
let isDownloadingPatches = $state(false);
let patchesError = $state("");
let patchManifest = $state<PatchManifestResponse | null>(null);
let patchStatus = $state<PatchStatusResponse | null>(null);

// ── Domain whitelist bypass ──
let whitelistStatus = $state<WhitelistStatusResponse | null>(null);
let connectMode = $state<ConnectMode>("raw");
let hostsDomain = $state("");
let isApplyingWhitelist = $state(false);
let whitelistError = $state("");

// ── Launcher config (server address etc.) ──
let serverAddress = $state("");
let serverAddressLoaded = $state(false);

// ── RUI archive editor ──
let ruiStatus = $state<RuiStatusResponse | null>(null);
let ruiConfig = $state<RuiConfig>(defaultRuiConfig());
let ruiBusy = $state(false);
let ruiError = $state("");
let ruiPreview = $state<{ changes: string[]; warnings: string[] } | null>(null);
let ruiLastReport = $state<RuiEditReport | null>(null);

function defaultRuiConfig(): RuiConfig {
  return {
    title: null,
    auto_connect: {
      enabled: false,
      ip: "",
      port: 22005,
      name: "AutoConnect",
      delay_ms: 2000,
    },
    server_list: { override: false, servers: [] },
    colors: {},
    texts: [],
    images: {},
  };
}

// ── Launch path selection ──
let launchTarget = $state<LaunchTarget>("updater");
let launchDiagnostic = $state<LaunchDiagnostic | null>(null);
let launchTargetError = $state("");

// ── Launch ──
let isLaunching = $state(false);
let statusMessage = $state("");
let launchPhase = $state(0);
let launchProgress = $state(0);

const launchPhases: LaunchPhase[] = [
  { label: "Проверка файлов", icon: "check" },
  { label: "Скачивание обновлений", icon: "download" },
  { label: "Загрузка ресурсов", icon: "cache" },
  { label: "Патч updater.exe", icon: "patch" },
  { label: "Запуск GTA V", icon: "play" },
];

// ── Load rage dir ──
async function loadRageDir() {
  try {
    const path = await invoke<string>("get_rage_dir");
    rageDir = path;
    clientReady = !!path;
  } catch (e) {
    handleError(e, "loadRageDir");
    clientReady = false;
  }
}

async function autoFindRageDir(playClickSound?: () => void) {
  playClickSound?.();
  try {
    const path = await invoke<string | null>("auto_find_rage_dir");
    if (path) {
      rageDir = path;
      clientReady = true;
      statusMessage = "RAGE:MP найден автоматически ✓";
      setTimeout(() => { statusMessage = ""; }, TIMING.statusMessageTimeout);
    } else {
      statusMessage = "RAGE:MP не найден автоматически";
      setTimeout(() => { statusMessage = ""; }, TIMING.statusMessageTimeout);
    }
  } catch (e) {
    handleError(e, "autoFindRageDir");
    statusMessage = "Автопоиск недоступен";
    setTimeout(() => { statusMessage = ""; }, TIMING.statusMessageTimeout);
  }
}

async function selectRageDir(playClickSound: () => void) {
  playClickSound();
  try {
    // Must be a DIRECTORY picker. This used to be a file dialog filtered to
    // `.exe`, so people ended up selecting things like `uninstall.exe` and the
    // launcher stored a file path as the client folder.
    const selected = await open({
      directory: true,
      multiple: false,
      title: "Выберите папку RAGE:MP",
    });
    if (!selected) return;

    const res = await invoke<{ path: string; valid: boolean }>("set_rage_dir", {
      path: selected,
    });

    // Trust the backend's verdict — never assume success.
    rageDir = res.path;
    clientReady = res.valid;

    if (res.valid) {
      statusMessage = "Путь к RAGE:MP сохранён ✓";
    } else {
      statusMessage =
        "В этой папке нет GTA5.exe или ragemp_v.exe — выберите папку RAGE:MP";
    }
    setTimeout(() => { statusMessage = ""; }, TIMING.statusMessageTimeout + 1500);
  } catch (e) {
    handleError(e, "selectRageDir");
  }
}

// ── Check + Download Client Files ──
async function checkAndDownloadClient(playClickSound: () => void): Promise<boolean> {
  playClickSound();

  try {
    // Check files
    isCheckingFiles = true;
    const result = await invoke<{ valid: boolean; skipped?: boolean; missing: unknown[]; outdated: unknown[]; total_download_size: number }>("check_client_files");
    isCheckingFiles = false;

    if (result.valid) {
      if (result.skipped) {
        statusMessage = "Проверка файлов пропущена (нет манифеста) — запуск установленного клиента";
      } else {
        statusMessage = "Файлы клиента актуальны ✓";
      }
      setTimeout(() => { statusMessage = ""; }, TIMING.statusMessageTimeout);
      return true;
    }

    // Download needed files
    isDownloadingClient = true;
    clientDownloadPercent = 0;
    clientDownloadFile = "";
    clientDownloadError = "";

    const unlisten = await listen<DownloadProgressPayload & { current_file: string }>(
      "client-download-progress",
      (event) => {
        const { percent, current_file } = event.payload;
        clientDownloadPercent = percent;
        clientDownloadFile = current_file;
      }
    );

    await invoke("download_client_files");
    unlisten();

    isDownloadingClient = false;
    clientDownloadPercent = 100;
    statusMessage = "Файлы клиента обновлены ✓";
    setTimeout(() => { statusMessage = ""; }, TIMING.statusMessageTimeout);
    return true;
  } catch (e) {
    isCheckingFiles = false;
    isDownloadingClient = false;
    handleError(e, "checkAndDownloadClient");
    clientDownloadError = String(e);
    return false;
  }
}

// ── Download RAGE:MP ──
async function downloadRagemp(playClickSound: () => void): Promise<boolean> {
  playClickSound();

  try {
    isDownloadingRagemp = true;
    ragempDownloadPercent = 0;
    ragempDownloadPhase = "download";
    ragempDownloadError = "";

    const unlisten = await listen<{
      phase: "download" | "extract";
      percent: number;
      downloaded?: number;
      total?: number;
      filename?: string;
    }>("ragemp-download-progress", (event) => {
      ragempDownloadPhase = event.payload.phase;
      ragempDownloadPercent = event.payload.percent;
    });

    const result = await invoke<{ success: boolean; rage_dir: string }>("download_ragemp");
    unlisten();

    if (result.success && result.rage_dir) {
      rageDir = result.rage_dir;
      clientReady = true;
    }

    isDownloadingRagemp = false;
    ragempDownloadPercent = 100;
    statusMessage = "RAGE:MP установлен ✓";
    setTimeout(() => { statusMessage = ""; }, TIMING.statusMessageTimeout);
    return true;
  } catch (e) {
    isDownloadingRagemp = false;
    handleError(e, "downloadRagemp");
    ragempDownloadError = String(e);
    return false;
  }
}

// ── Check + Download Server Cache ──
async function checkAndDownloadCache(playClickSound: () => void): Promise<boolean> {
  playClickSound();

  try {
    isCheckingCache = true;
    const result = await invoke<{ needed: boolean; skipped?: boolean; reason: string }>("check_server_cache");
    isCheckingCache = false;

    if (!result.needed) {
      if (result.skipped) {
        statusMessage = "Проверка серверного кэша пропущена (нет endpoint) — доверяем установленному";
      } else {
        statusMessage = "Серверный кэш актуален ✓";
      }
      setTimeout(() => { statusMessage = ""; }, TIMING.statusMessageTimeout);
      return true;
    }

    // Download cache
    isDownloadingCache = true;
    cachePercent = 0;
    cachePhase = "download";
    cacheError = "";

    const unlisten = await listen<{
      phase: "download" | "extract";
      percent: number;
      downloaded?: number;
      total?: number;
      extracted?: number;
      total_files?: number;
    }>("cache-download-progress", (event) => {
      cachePhase = event.payload.phase;
      cachePercent = event.payload.percent;
    });

    await invoke("download_server_cache");
    unlisten();

    isDownloadingCache = false;
    cachePercent = 100;
    statusMessage = "Серверный кэш загружен ✓";
    setTimeout(() => { statusMessage = ""; }, TIMING.statusMessageTimeout);
    return true;
  } catch (e) {
    isCheckingCache = false;
    isDownloadingCache = false;
    handleError(e, "checkAndDownloadCache");
    cacheError = String(e);
    return false;
  }
}

// ── Download Patches ──
async function downloadPatches(playClickSound: () => void): Promise<boolean> {
  playClickSound();

  try {
    isDownloadingPatches = true;
    patchesError = "";
    // Resolves the server manifest, falling back to the built-in table
    // ported from tiny-ragemp-launcher.
    patchManifest = await invoke<PatchManifestResponse>("get_patch_manifest");
    isDownloadingPatches = false;

    const src = patchManifest.source === "builtin" ? "встроенные" : "с сервера";
    statusMessage = `Патчи (${src}): ${patchManifest.count} ✓`;
    setTimeout(() => { statusMessage = ""; }, TIMING.statusMessageTimeout);
    return true;
  } catch (e) {
    isDownloadingPatches = false;
    handleError(e, "downloadPatches");
    patchesError = String(e);
    return false;
  }
}

// ── Patch + whitelist diagnostics ──
async function loadPatchStatus() {
  try {
    patchStatus = await invoke<PatchStatusResponse>("check_patch_status");
  } catch (e) {
    handleError(e, "loadPatchStatus");
    patchStatus = null;
  }
}

async function loadWhitelistStatus() {
  try {
    const status = await invoke<WhitelistStatusResponse>("get_whitelist_status");
    whitelistStatus = status;
    connectMode = status.connect_mode ?? "raw";
    hostsDomain = status.hosts_domain ?? "";
  } catch (e) {
    handleError(e, "loadWhitelistStatus");
    whitelistStatus = null;
  }
}

/** Switch the connect method (raw IP vs. whitelisted domain via hosts file). */
async function setConnectMode(mode: ConnectMode, domain?: string) {
  try {
    await invoke("set_connect_mode", {
      mode,
      hostsDomain: domain ?? hostsDomain,
    });
    connectMode = mode;
    if (domain !== undefined) hostsDomain = domain;
    statusMessage = mode === "hosts" ? "Режим: домен через hosts" : "Режим: прямой IP";
    setTimeout(() => { statusMessage = ""; }, TIMING.statusMessageTimeout);
    return true;
  } catch (e) {
    handleError(e, "setConnectMode");
    whitelistError = String(e);
    return false;
  }
}

/** Apply the hosts-file whitelist bypass (may raise a UAC prompt). */
async function applyWhitelistBypass(): Promise<boolean> {
  try {
    isApplyingWhitelist = true;
    whitelistError = "";

    const result = await invoke<{ success: boolean; domain: string; ip: string }>(
      "apply_whitelist_bypass"
    );

    isApplyingWhitelist = false;
    connectMode = "hosts";
    hostsDomain = result.domain;
    await loadWhitelistStatus();

    statusMessage = `Обход белого списка: ${result.domain} ✓`;
    setTimeout(() => { statusMessage = ""; }, TIMING.statusMessageTimeout + 1000);
    return true;
  } catch (e) {
    isApplyingWhitelist = false;
    handleError(e, "applyWhitelistBypass");
    whitelistError = String(e);
    statusMessage = "Обход не применён (нужны права администратора)";
    setTimeout(() => { statusMessage = ""; }, TIMING.statusMessageTimeout + 1000);
    return false;
  }
}

/**
 * Load the launcher config once, so the UI shows the *actual* server address
 * instead of a hardcoded one (the old label showed a leftover default port).
 */
async function loadLauncherConfig() {
  try {
    const cfg = await invoke<LauncherConfigResponse>("get_launcher_config");
    serverAddress = cfg.server_address ?? "";
    if (cfg.connect_mode) connectMode = cfg.connect_mode;
    if (cfg.hosts_domain !== undefined) hostsDomain = cfg.hosts_domain;
    if (cfg.launch_target) launchTarget = cfg.launch_target;
    serverAddressLoaded = true;
  } catch (e) {
    handleError(e, "loadLauncherConfig");
  }
}

/** Choose which binary the launcher runs. */
async function setLaunchTarget(target: LaunchTarget): Promise<boolean> {
  try {
    launchTargetError = "";
    const res = await invoke<{ target: LaunchTarget; binary_present: boolean | null }>(
      "set_launch_target",
      { target }
    );
    launchTarget = res.target;

    if (res.binary_present === false) {
      launchTargetError = target === "ragemp_v"
        ? "ragemp_v.exe не найден в папке RAGE:MP"
        : "updater.exe не найден в папке RAGE:MP";
    }

    await loadLaunchDiagnostic();
    statusMessage = target === "ragemp_v"
      ? "Путь: напрямую через ragemp_v.exe"
      : "Путь: через updater.exe";
    setTimeout(() => { statusMessage = ""; }, TIMING.statusMessageTimeout);
    return true;
  } catch (e) {
    handleError(e, "setLaunchTarget");
    launchTargetError = String(e);
    return false;
  }
}

/** Refresh the full launch diagnostic. */
async function loadLaunchDiagnostic() {
  try {
    launchDiagnostic = await invoke<LaunchDiagnostic>("diagnose_launch");
    if (launchDiagnostic) {
      launchTarget = launchDiagnostic.target;
    }
  } catch (e) {
    handleError(e, "loadLaunchDiagnostic");
    launchDiagnostic = null;
  }
}

/** Wipe the direct-connect registry values (troubleshooting). */
async function clearLaunchRegistry(): Promise<boolean> {
  try {
    await invoke("clear_launch_registry");
    await loadLaunchDiagnostic();
    statusMessage = "Значения реестра очищены";
    setTimeout(() => { statusMessage = ""; }, TIMING.statusMessageTimeout);
    return true;
  } catch (e) {
    handleError(e, "clearLaunchRegistry");
    return false;
  }
}

/**
 * Toggle the rui auto-connect injection. Off by default — injecting the
 * auto-connect block into index.rui has been reported to make the CEF
 * launcher UI show "404 error: File not found", so it stays opt-in.
 */
async function setRuiAutoConnect(enabled: boolean): Promise<boolean> {
  try {
    const res = await invoke<{ rui_auto_connect: boolean }>("set_rui_auto_connect", {
      enabled,
    });
    await loadLaunchDiagnostic();
    statusMessage = res.rui_auto_connect
      ? "Автоподключение RUI включено"
      : "Автоподключение RUI отключено";
    setTimeout(() => { statusMessage = ""; }, TIMING.statusMessageTimeout);
    return true;
  } catch (e) {
    handleError(e, "setRuiAutoConnect");
    return false;
  }
}

/** Validate + save the server address. Returns a warning message, or "". */
async function setServerAddress(address: string): Promise<string> {
  try {
    const res = await invoke<{
      server_address: string;
      host: string;
      port: number;
      rage_port_warning: boolean;
    }>("set_server_address", { address });

    serverAddress = res.server_address;
    await loadLaunchDiagnostic();

    if (res.rage_port_warning) {
      statusMessage = "Порт 30120 — нестандартный порт. Для RAGE:MP обычно 22005";
      setTimeout(() => { statusMessage = ""; }, TIMING.statusMessageTimeout + 2000);
      return "Порт 30120 — нестандартный порт, для RAGE:MP обычно 22005";
    }

    statusMessage = `Сервер: ${res.server_address} ✓`;
    setTimeout(() => { statusMessage = ""; }, TIMING.statusMessageTimeout);
    return "";
  } catch (e) {
    handleError(e, "setServerAddress");
    return String(e);
  }
}

// ── RUI archive editor ──
async function loadRuiStatus() {
  try {
    ruiStatus = await invoke<RuiStatusResponse>("get_rui_status");
  } catch (e) {
    handleError(e, "loadRuiStatus");
    ruiStatus = null;
  }
}

async function loadRuiConfig() {
  try {
    const cfg = await invoke<RuiConfig>("get_rui_config");
    ruiConfig = { ...defaultRuiConfig(), ...cfg };
  } catch (e) {
    handleError(e, "loadRuiConfig");
    ruiConfig = defaultRuiConfig();
  }
}

async function saveRuiConfig(): Promise<boolean> {
  try {
    await invoke("save_rui_config", { config: ruiConfig });
    return true;
  } catch (e) {
    handleError(e, "saveRuiConfig");
    ruiError = String(e);
    return false;
  }
}

/** Dry-run: report what the current config would change. */
async function previewRuiConfig(): Promise<boolean> {
  try {
    ruiBusy = true;
    ruiError = "";
    ruiPreview = await invoke<{ changes: string[]; warnings: string[] }>(
      "preview_rui_config",
      { config: ruiConfig }
    );
    ruiBusy = false;
    return true;
  } catch (e) {
    ruiBusy = false;
    handleError(e, "previewRuiConfig");
    ruiError = String(e);
    ruiPreview = null;
    return false;
  }
}

/** Apply the config to index.rui (backs the archive up first). */
async function applyRuiConfig(): Promise<boolean> {
  try {
    ruiBusy = true;
    ruiError = "";
    ruiLastReport = await invoke<RuiEditReport>("apply_rui_config", {
      config: ruiConfig,
    });
    ruiBusy = false;
    await saveRuiConfig();
    await loadRuiStatus();

    const n = ruiLastReport.entries_changed;
    statusMessage = n > 0 ? `index.rui обновлён: ${n} файл(ов)` : "index.rui без изменений";
    setTimeout(() => { statusMessage = ""; }, TIMING.statusMessageTimeout + 1000);
    return true;
  } catch (e) {
    ruiBusy = false;
    handleError(e, "applyRuiConfig");
    ruiError = String(e);
    return false;
  }
}

/** Restore rui\index.rui from the backup made by the archive editor. */
async function restoreRui(): Promise<boolean> {
  try {
    const res = await invoke<{ success: boolean; restored: boolean }>("restore_rui");
    await loadRuiStatus();
    statusMessage = res.restored ? "index.rui восстановлен из бэкапа ✓" : "Бэкап index.rui не найден";
    setTimeout(() => { statusMessage = ""; }, TIMING.statusMessageTimeout);
    return res.restored;
  } catch (e) {
    handleError(e, "restoreRui");
    return false;
  }
}

/** Update a single field of the RUI config (reactive-friendly helper). */
function setRuiField(patch: Partial<RuiConfig>) {
  ruiConfig = { ...ruiConfig, ...patch };
}

function setAutoConnectField(patch: Partial<RuiConfig["auto_connect"]>) {
  ruiConfig = {
    ...ruiConfig,
    auto_connect: { ...ruiConfig.auto_connect, ...patch },
  };
}

/** Remove the hosts entries and fall back to the raw-IP method. */
async function removeWhitelistBypass(): Promise<boolean> {
  try {
    isApplyingWhitelist = true;
    whitelistError = "";
    await invoke("remove_whitelist_bypass");
    isApplyingWhitelist = false;
    connectMode = "raw";
    await loadWhitelistStatus();
    statusMessage = "Обход снят, режим: прямой IP";
    setTimeout(() => { statusMessage = ""; }, TIMING.statusMessageTimeout);
    return true;
  } catch (e) {
    isApplyingWhitelist = false;
    handleError(e, "removeWhitelistBypass");
    whitelistError = String(e);
    return false;
  }
}

// ── Main Launch Flow ──
async function handlePlay(playClickSound: () => void, serverOnline: boolean, setDiscordRpcPlaying: () => Promise<void>) {
  playClickSound();
  if (isLaunching) return;

  // If RAGE:MP not found, try to download it
  if (!clientReady) {
    statusMessage = "RAGE:MP не найден. Скачиваем...";
    const downloaded = await downloadRagemp(playClickSound);
    if (!downloaded) {
      statusMessage = "RAGE:MP не установлен. Укажите путь в настройках.";
      setTimeout(() => { statusMessage = ""; }, TIMING.statusMessageTimeout + 1000);
      return;
    }
  }

  isLaunching = true;

  try {
    // Phase 1: Check + download client files
    launchPhase = 1;
    launchProgress = 5;
    const filesOk = await checkAndDownloadClient(() => {});
    if (!filesOk) {
      throw new Error("Client file check/download failed");
    }
    launchProgress = 25;

    // Phase 2: Check + download server cache
    launchPhase = 2;
    launchProgress = 30;
    const cacheOk = await checkAndDownloadCache(() => {});
    if (!cacheOk) {
      throw new Error("Cache check/download failed");
    }
    launchProgress = 60;

    // Phase 3: Resolve the patch table (server manifest or built-in)
    launchPhase = 3;
    launchProgress = 65;
    const patchesOk = await downloadPatches(() => {});
    if (!patchesOk) {
      throw new Error("Patch download failed");
    }
    launchProgress = 80;

    // Phase 4: Launch — registry write, patch updater.exe, (rui auto-connect), spawn
    launchPhase = 4;
    launchProgress = 85;
    const result = await invoke<{
      success: boolean;
      pid: number;
      connect_host?: string;
      connect_mode?: string;
      patches_applied?: number;
      rui_auto_connect?: boolean;
    }>("launch_patched");
    launchProgress = 100;

    // Done
    launchPhase = 0;
    launchProgress = 0;
    isLaunching = false;
    statusMessage = "GTA V запущен ✓";
    setDiscordRpcPlaying();
    // Refresh the diagnostics so the settings panel reflects the new state.
    loadPatchStatus();
    loadWhitelistStatus();
    setTimeout(() => { statusMessage = ""; }, TIMING.statusMessageTimeout);
  } catch (error) {
    launchPhase = 0;
    launchProgress = 0;
    isLaunching = false;

    const errMsg = error instanceof Error ? error.message : String(error);
    if (errMsg.includes("CLIENT_NOT_FOUND")) {
      statusMessage = "RAGE:MP не найден. Укажите путь в настройках.";
    } else {
      statusMessage = `Ошибка: ${errMsg}`;
    }
    handleError(error, "handlePlay");
    setTimeout(() => { statusMessage = ""; }, TIMING.statusMessageTimeout + 1000);
  }
}

export function getGameStore() {
  return {
    get rageDir() { return rageDir; },
    get clientReady() { return clientReady; },
    get isCheckingFiles() { return isCheckingFiles; },
    get isDownloadingClient() { return isDownloadingClient; },
    get clientDownloadPercent() { return clientDownloadPercent; },
    get clientDownloadFile() { return clientDownloadFile; },
    get clientDownloadError() { return clientDownloadError; },
    get isDownloadingRagemp() { return isDownloadingRagemp; },
    get ragempDownloadPercent() { return ragempDownloadPercent; },
    get ragempDownloadPhase() { return ragempDownloadPhase; },
    get ragempDownloadError() { return ragempDownloadError; },
    get isCheckingCache() { return isCheckingCache; },
    get isDownloadingCache() { return isDownloadingCache; },
    get cachePercent() { return cachePercent; },
    get cachePhase() { return cachePhase; },
    get cacheError() { return cacheError; },
    get isDownloadingPatches() { return isDownloadingPatches; },
    get patchesError() { return patchesError; },
    get patchManifest() { return patchManifest; },
    get patchStatus() { return patchStatus; },
    get whitelistStatus() { return whitelistStatus; },
    get connectMode() { return connectMode; },
    get hostsDomain() { return hostsDomain; },
    get isApplyingWhitelist() { return isApplyingWhitelist; },
    get whitelistError() { return whitelistError; },
    get serverAddress() { return serverAddress; },
    get serverAddressLoaded() { return serverAddressLoaded; },
    get ruiStatus() { return ruiStatus; },
    get ruiConfig() { return ruiConfig; },
    get ruiBusy() { return ruiBusy; },
    get ruiError() { return ruiError; },
    get ruiPreview() { return ruiPreview; },
    get ruiLastReport() { return ruiLastReport; },
    get launchTarget() { return launchTarget; },
    get launchDiagnostic() { return launchDiagnostic; },
    get launchTargetError() { return launchTargetError; },
    get launching() { return isLaunching; },
    get statusMessage() { return statusMessage; },
    get launchPhase() { return launchPhase; },
    get launchProgress() { return launchProgress; },
    get launchPhases() { return launchPhases; },
    loadRageDir,
    autoFindRageDir,
    selectRageDir,
    checkAndDownloadClient,
    checkAndDownloadCache,
    downloadRagemp,
    downloadPatches,
    loadPatchStatus,
    loadWhitelistStatus,
    setConnectMode,
    applyWhitelistBypass,
    removeWhitelistBypass,
    loadLauncherConfig,
    setLaunchTarget,
    loadLaunchDiagnostic,
    clearLaunchRegistry,
    setRuiAutoConnect,
    setServerAddress,
    loadRuiStatus,
    loadRuiConfig,
    saveRuiConfig,
    previewRuiConfig,
    applyRuiConfig,
    restoreRui,
    setRuiField,
    setAutoConnectField,
    handlePlay,
  };
}
