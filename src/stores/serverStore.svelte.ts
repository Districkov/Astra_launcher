/**
 * serverStore.svelte.ts — Статус сервера, пинг, уведомления
 * Svelte 5 runes-based store
 */
import { invoke } from "@tauri-apps/api/core";
import { TIMING } from "../constants";
import { handleError } from "../errorHandling";
import type { Notification, ServerStatusResponse, ServerPingResponse } from "../types";

// ── Состояние сервера ──
let serverOnline = $state(false);
let serverPlayers = $state(0);
let serverMaxPlayers = $state(0);
let serverLoading = $state(true);
let serverPing = $state(0);
let serverInitialized = $state(false);
let prevServerOnline = $state<boolean | null>(null);
let offlineMode = $state(false);

// ── Уведомления ──
let notifications = $state<Notification[]>([]);
let notificationId = 0;

function addNotification(message: string, type: Notification["type"] = "info") {
  const id = ++notificationId;
  // Push + splice instead of spread — avoids re-creating the entire array
  notifications.push({ id: String(id), message, type, timestamp: Date.now() });
  notifications = notifications; // trigger reactivity
  setTimeout(() => {
    const idx = notifications.findIndex(n => n.id === String(id));
    if (idx !== -1) {
      notifications.splice(idx, 1);
      notifications = notifications; // trigger reactivity
    }
  }, TIMING.notificationTimeout);
}

// ── Загрузка статуса сервера ──
async function loadServerStatus() {
  if (!serverInitialized) serverLoading = true;
  try {
    const status = await invoke<ServerStatusResponse>("get_server_status");
    const wasOnline = prevServerOnline;
    serverOnline = status.online;
    serverPlayers = status.players;
    serverMaxPlayers = status.max_players;
    prevServerOnline = status.online;
    serverInitialized = true;
    offlineMode = false;

    if (wasOnline !== null && wasOnline !== status.online) {
      if (status.online) {
        addNotification("🟢 Сервер снова онлайн!", "success");
      } else {
        addNotification("🔴 Сервер перешёл в офлайн", "error");
      }
    }
  } catch (e) {
    handleError(e, "loadServerStatus");
    serverOnline = false;
    if (!serverInitialized) {
      offlineMode = true;
      addNotification("📡 Нет подключения к интернету", "error");
    }
  } finally {
    serverLoading = false;
  }
}

async function loadServerPing() {
  try {
    const result = await invoke<ServerPingResponse>("get_server_ping");
    serverPing = result.ping || 0;
  } catch (e) {
    handleError(e, "loadServerPing");
    serverPing = 0;
  }
}

async function refreshServerStatus() {
  serverLoading = true;
  await Promise.all([loadServerStatus(), loadServerPing()]);
}

// ── Экспорт ──
export function getServerStore() {
  return {
    get online() { return serverOnline; },
    get players() { return serverPlayers; },
    get maxPlayers() { return serverMaxPlayers; },
    get loading() { return serverLoading; },
    get ping() { return serverPing; },
    get initialized() { return serverInitialized; },
    get offline() { return offlineMode; },
    get notifications() { return notifications; },
    addNotification,
    loadServerStatus,
    loadServerPing,
    refreshServerStatus,
  };
}
