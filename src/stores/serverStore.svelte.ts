/**
 * serverStore.svelte.ts — Статус сервера, пинг, обновления
 * Svelte 5 runes-based store
 */
import { invoke } from "@tauri-apps/api/core";
import { TIMING } from "../constants";

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
let notifications = $state<{ id: number; message: string; type: string }[]>([]);
let notificationId = 0;

function addNotification(message: string, type = "info") {
  const id = ++notificationId;
  notifications = [...notifications, { id, message, type }];
  setTimeout(() => {
    notifications = notifications.filter(n => n.id !== id);
  }, TIMING.notificationTimeout);
}

// ── Загрузка статуса сервера ──
async function loadServerStatus() {
  if (!serverInitialized) serverLoading = true;
  try {
    const status = await invoke("get_server_status") as any;
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
    const result = await invoke("get_server_ping") as any;
    serverPing = result.ping || 0;
  } catch (e) {
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
