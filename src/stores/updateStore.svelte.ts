/**
 * updateStore.svelte.ts — Автообновление (tauri-plugin-updater)
 * Svelte 5 runes-based store
 */
import { invoke } from "@tauri-apps/api/core";
import { relaunch } from "@tauri-apps/plugin-process";
import { handleError } from "../errorHandling";
import type { UpdateInfo, UpdateEventPayload } from "../types";

let updateAvailable = $state(false);
let updateInfo = $state<UpdateInfo>({ current_version: "", latest_version: "", release_notes: "" });
let updateChecked = $state(false);
let updateChecking = $state(false);
let updateDownloading = $state(false);
let updateDownloadPercent = $state(0);
let updateDownloaded = $state(0);
let updateTotal = $state(0);
let updateError = $state("");
let showUpdateModal = $state(false);
let pendingUpdate: Awaited<ReturnType<typeof import('@tauri-apps/plugin-updater').check>> | null = $state(null);

// Lazy getter to avoid circular dependency at module load
function getServerStore() {
  return import("./serverStore.svelte").then(m => m.getServerStore());
}

async function checkUpdates() {
  if (updateChecking) return;
  updateChecking = true;
  try {
    const { check } = await import('@tauri-apps/plugin-updater');
    const update = await check();
    updateChecked = true;
    if (update) {
      updateAvailable = true;
      pendingUpdate = update;
      updateInfo = {
        current_version: update.currentVersion,
        latest_version: update.version,
        release_notes: update.body || "",
      };
      const server = await getServerStore();
      server.addNotification(`🔄 Доступно обновление: v${update.version}`, "update");
    }
  } catch (e) {
    handleError(e, "checkUpdates");
    updateChecked = true;
  } finally {
    updateChecking = false;
  }
}

async function startUpdateDownload(playClickSound: () => void) {
  playClickSound();
  if (!pendingUpdate) return;
  updateDownloading = true;
  updateDownloadPercent = 0;
  updateError = "";
  updateDownloaded = 0;
  updateTotal = 0;

  try {
    let downloaded = 0;
    let total = 0;
    await pendingUpdate.downloadAndInstall((event: UpdateEventPayload) => {
      switch (event.event) {
        case 'Started':
          total = event.data.contentLength || 0;
          updateTotal = total;
          break;
        case 'Progress':
          downloaded += event.data.chunkLength || 0;
          updateDownloaded = downloaded;
          updateDownloadPercent = total > 0 ? Math.round((downloaded / total) * 100) : 0;
          break;
        case 'Finished':
          updateDownloadPercent = 100;
          break;
      }
    });
    const server = await getServerStore();
    server.addNotification("✅ Обновление установлено! Перезапуск...", "success");
    // Перепроверяем FiveM путь перед перезапуском
    const { getFivemStore } = await import("./fivemStore.svelte");
    await getFivemStore().loadFivemPath();
    setTimeout(() => {
      relaunch();
    }, 1000);
  } catch (e) {
    handleError(e, "startUpdateDownload");
    updateError = String(e);
    updateDownloading = false;
  }
}

function openUpdateModal(playClickSound: () => void) {
  playClickSound();
  showUpdateModal = true;
}

function closeUpdateModal(playClickSound: () => void) {
  playClickSound();
  showUpdateModal = false;
}

function setCurrentVersion(version: string) {
  updateInfo = { ...updateInfo, current_version: version };
}

export function getUpdateStore() {
  return {
    get available() { return updateAvailable; },
    get info() { return updateInfo; },
    get checked() { return updateChecked; },
    get checking() { return updateChecking; },
    get downloading() { return updateDownloading; },
    get downloadPercent() { return updateDownloadPercent; },
    get downloaded() { return updateDownloaded; },
    get total() { return updateTotal; },
    get error() { return updateError; },
    get showModal() { return showUpdateModal; },
    set showModal(v: boolean) { showUpdateModal = v; },
    checkUpdates,
    startUpdateDownload,
    openUpdateModal,
    closeUpdateModal,
    setCurrentVersion,
  };
}
