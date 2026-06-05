/**
 * updateStore.svelte.ts — Автообновление (tauri-plugin-updater)
 * Svelte 5 runes-based store
 */
import { invoke } from "@tauri-apps/api/core";
import { relaunch } from "@tauri-apps/plugin-process";
import { getServerStore } from "./serverStore.svelte";
import { getFivemStore } from "./fivemStore.svelte";

let updateAvailable = $state(false);
let updateInfo = $state({ current_version: "", latest_version: "", release_notes: "" });
let updateChecked = $state(false);
let updateDownloading = $state(false);
let updateDownloadPercent = $state(0);
let updateDownloaded = $state(0);
let updateTotal = $state(0);
let updateError = $state("");
let showUpdateModal = $state(false);
let pendingUpdate: any = $state(null);

const { addNotification } = getServerStore();

async function checkUpdates() {
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
      addNotification(`🔄 Доступно обновление: v${update.version}`, "update");
    }
  } catch (e) {
    console.error("Update check failed:", e);
    updateChecked = true;
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
    await pendingUpdate.downloadAndInstall((event: any) => {
      switch (event.event) {
        case 'Started':
          total = event.data.contentLength || 0;
          updateTotal = total;
          break;
        case 'Progress':
          downloaded += event.data.chunkLength;
          updateDownloaded = downloaded;
          updateDownloadPercent = total > 0 ? Math.round((downloaded / total) * 100) : 0;
          break;
        case 'Finished':
          updateDownloadPercent = 100;
          break;
      }
    });
    addNotification("✅ Обновление установлено! Перезапуск...", "success");
    // Перепроверяем FiveM путь перед перезапуском
    const fivem = getFivemStore();
    await fivem.loadFivemPath();
    setTimeout(() => {
      relaunch();
    }, 1000);
  } catch (e) {
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

export function getUpdateStore() {
  return {
    get available() { return updateAvailable; },
    get info() { return updateInfo; },
    get checked() { return updateChecked; },
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
  };
}
