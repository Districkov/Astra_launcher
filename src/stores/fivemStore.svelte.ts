/**
 * fivemStore.svelte.ts — Путь к FiveM, скачивание, запуск
 * Svelte 5 runes-based store
 */
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { listen } from "@tauri-apps/api/event";
import { handleError } from "../errorHandling";
import { fmtSize, TIMING } from "../constants";
import type { DownloadProgressPayload, PrecacheProgressPayload, PrecacheCheckResponse, LaunchPhase } from "../types";

// Lazy getter to avoid circular dependency at module load
function getServerStore() {
  return import("./serverStore.svelte").then(m => m.getServerStore());
}

// ── FiveM путь ──
let fivemPath = $state("");
let fivemFound = $state(false);

// ── Скачивание FiveM ──
let isDownloading = $state(false);
let downloadPercent = $state(0);
let downloadSize = $state("");
let downloadError = $state("");

// ── Предзагрузка кеша ──
let precacheNeeded = $state(false);
let precacheDownloading = $state(false);
let precacheExtracting = $state(false);
let precachePercent = $state(0);
let precacheDownloaded = $state("");
let precacheTotal = $state("");

// ── Запуск игры ──
let isLaunching = $state(false);
let statusMessage = $state("");
let launchPhase = $state(0);
let launchProgress = $state(0);

const launchPhases: LaunchPhase[] = [
  { label: "Проверка клиента", icon: "check" },
  { label: "Подключение к серверу", icon: "connect" },
  { label: "Загрузка ресурсов", icon: "download" },
  { label: "Запуск FiveM", icon: "play" },
];

// ── Загрузка пути FiveM ──
async function loadFivemPath() {
  try {
    const path = await invoke<string>("get_fivem_path");
    fivemPath = path;
    fivemFound = !!path;
  } catch (e) {
    handleError(e, "loadFivemPath");
    fivemFound = false;
  }
}

async function autoFindFivem(playClickSound?: () => void) {
  playClickSound?.();
  try {
    const path = await invoke<string | null>("auto_find_fivem");
    if (path) {
      fivemPath = path;
      fivemFound = true;
      statusMessage = "FiveM найден автоматически ✓";
      setTimeout(() => { statusMessage = ""; }, TIMING.statusMessageTimeout);
    } else {
      statusMessage = "FiveM не найден автоматически";
      setTimeout(() => { statusMessage = ""; }, TIMING.statusMessageTimeout);
    }
  } catch (e) {
    handleError(e, "autoFindFivem");
    statusMessage = "Автопоиск недоступен";
    setTimeout(() => { statusMessage = ""; }, TIMING.statusMessageTimeout);
  }
}

async function selectFivemPath(playClickSound: () => void) {
  playClickSound();
  try {
    const selected = await open({
      multiple: false,
      filters: [{ name: "FiveM", extensions: ["exe"] }],
    });
    if (selected) {
      await invoke("set_fivem_path", { path: selected });
      fivemPath = selected;
      fivemFound = true;
      statusMessage = "Путь к FiveM сохранён ✓";
      setTimeout(() => { statusMessage = ""; }, TIMING.statusMessageTimeout);
    }
  } catch (e) {
    handleError(e, "selectFivemPath");
  }
}

// ── Скачивание FiveM ──
async function downloadAndInstall(playClickSound: () => void) {
  playClickSound();
  if (isDownloading) return;
  isDownloading = true;
  downloadPercent = 0;
  downloadSize = "";
  downloadError = "";
  try {
    const unlisten = await listen<DownloadProgressPayload>("download-progress", (event) => {
      const { downloaded, total, percent } = event.payload;
      downloadPercent = percent;
      downloadSize = total > 0 ? `${fmtSize(downloaded)} / ${fmtSize(total)}` : fmtSize(downloaded);
    });
    await invoke("download_fivem");
    unlisten();
    downloadPercent = 100;
    downloadSize = "Запуск установщика…";
    await invoke("launch_fivem_installer");
    statusMessage = "Установщик запущен. Дождитесь установки, затем нажмите «Проверить».";
    const pollStart = Date.now();
    const POLL_TIMEOUT = TIMING.installerPollTimeout;
    const poll = setInterval(async () => {
      try {
        if (Date.now() - pollStart > POLL_TIMEOUT) {
          clearInterval(poll);
          isDownloading = false;
          statusMessage = "Время ожидания истекло. Нажмите «Проверить» вручную.";
          setTimeout(() => { statusMessage = ""; }, TIMING.statusMessageTimeout + 2000);
          return;
        }
        const installed = await invoke<boolean>("check_fivem_installed");
        if (installed) {
          clearInterval(poll);
          await loadFivemPath();
          isDownloading = false;
          statusMessage = "FiveM установлен ✓";
          setTimeout(() => { statusMessage = ""; }, TIMING.statusMessageTimeout);
        }
      } catch (e) {
        handleError(e, "downloadAndInstall.poll");
      }
    }, 3000);
  } catch (error) {
    handleError(error, "downloadAndInstall");
    downloadError = `${error}`;
    isDownloading = false;
  }
}

async function checkFivemAgain(playClickSound: () => void) {
  playClickSound();
  await loadFivemPath();
}

// ── Запуск игры ──
async function handlePlay(playClickSound: () => void, serverOnline: boolean, setDiscordRpcPlaying: () => Promise<void>) {
  playClickSound();
  if (isLaunching) return;

  // Check if FiveM is found before attempting launch
  if (!fivemFound) {
    statusMessage = "FiveM не найден. Укажите путь в настройках.";
    setTimeout(() => { statusMessage = ""; }, TIMING.statusMessageTimeout + 1000);
    return;
  }

  isLaunching = true;
  launchPhase = 1;
  launchProgress = 5;

  try {
    const check = await invoke<PrecacheCheckResponse>("check_precache_needed");
    precacheNeeded = check.needed;

    if (check.needed) {
      launchPhase = 3;
      precacheDownloading = true;
      precachePercent = 0;

      const unlisten = await listen<PrecacheProgressPayload>("precache-progress", (event) => {
        const { phase, downloaded, total, percent } = event.payload;
        if (phase === "download") {
          precacheDownloading = true;
          precacheExtracting = false;
          precachePercent = percent;
          precacheDownloaded = fmtSize(downloaded ?? 0);
          precacheTotal = (total ?? 0) > 0 ? fmtSize(total ?? 0) : "";
          launchProgress = 5 + Math.round(percent * 0.35);
        } else if (phase === "extract") {
          precacheDownloading = false;
          precacheExtracting = true;
          precachePercent = percent;
          launchProgress = 40 + Math.round(percent * 0.1);
        }
      });

      try {
        await invoke("precache_server_files");
        precacheNeeded = false;
      } catch (error) {
        const server = await getServerStore();
        server.addNotification("⚠️ Предзагрузка не удалась: " + error, "error");
      }

      unlisten();
      precacheDownloading = false;
      precacheExtracting = false;
    }

    launchProgress = 50;
  } catch (e) {
    handleError(e, "handlePlay.precache");
    launchProgress = 25;
  }

  launchPhase = 2;
  launchProgress = 55;
  try {
    await invoke("launch_game");
    launchProgress = 70;
  } catch (error: unknown) {
    launchPhase = 0;
    launchProgress = 0;
    isLaunching = false;
    precacheDownloading = false;
    precacheExtracting = false;
    const errMsg = error instanceof Error ? error.message : String(error);
    if (errMsg === "FIVEM_NOT_FOUND") {
      statusMessage = "FiveM не найден. Укажите путь в настройках.";
    } else {
      statusMessage = `Ошибка: ${errMsg}`;
    }
    handleError(error, "handlePlay.launch");
    setTimeout(() => { statusMessage = ""; }, TIMING.statusMessageTimeout + 1000);
    return;
  }

  // Real launch phases — no fake sleep delays
  launchPhase = 3;
  launchProgress = 75;
  launchPhase = 4;
  launchProgress = 100;

  launchPhase = 0;
  launchProgress = 0;
  isLaunching = false;
  precacheDownloading = false;
  precacheExtracting = false;
  statusMessage = "FiveM запущен ✓";
  setDiscordRpcPlaying();
  setTimeout(() => { statusMessage = ""; }, TIMING.statusMessageTimeout);
}

export function getFivemStore() {
  return {
    get path() { return fivemPath; },
    get found() { return fivemFound; },
    get downloading() { return isDownloading; },
    get downloadPercent() { return downloadPercent; },
    get downloadSize() { return downloadSize; },
    get downloadError() { return downloadError; },
    get precacheNeeded() { return precacheNeeded; },
    get precacheDownloading() { return precacheDownloading; },
    get precacheExtracting() { return precacheExtracting; },
    get precachePercent() { return precachePercent; },
    get precacheDownloaded() { return precacheDownloaded; },
    get precacheTotal() { return precacheTotal; },
    get launching() { return isLaunching; },
    get statusMessage() { return statusMessage; },
    get launchPhase() { return launchPhase; },
    get launchProgress() { return launchProgress; },
    get launchPhases() { return launchPhases; },
    loadFivemPath,
    autoFindFivem,
    selectFivemPath,
    downloadAndInstall,
    checkFivemAgain,
    handlePlay,
  };
}
