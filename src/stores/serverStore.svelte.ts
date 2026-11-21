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
/** How the server was judged reachable: "http" | "tcp" | "none". */
let serverReachability = $state("none");
/** Whether the server exposes stats endpoints at all. */
let serverHasStats = $state(false);

/**
 * Guards against overlapping polls.
 *
 * The status check can take as long as its timeout; if the poll interval fires
 * again before the previous call resolves, requests pile up and the UI looks
 * permanently stuck on "checking…".
 */
let statusInFlight = false;
let pingInFlight = false;

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
  // Skip if a poll is already running — prevents pile-up when the check
  // takes longer than the interval.
  if (statusInFlight) return;
  statusInFlight = true;

  if (!serverInitialized) serverLoading = true;
  try {
    const status = await invoke<ServerStatusResponse>("get_server_status");
    const wasOnline = prevServerOnline;
    serverOnline = status.online;
    serverPlayers = status.players;
    serverMaxPlayers = status.max_players;
    serverReachability = status.reachability ?? (status.online ? "http" : "none");
    serverHasStats = status.has_stats ?? false;
    prevServerOnline = status.online;
    serverInitialized = true;
    offlineMode = false;

    // Only announce a genuine transition, and only once we have a baseline.
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
    serverReachability = "none";
    if (!serverInitialized) {
      offlineMode = true;
      addNotification("📡 Нет подключения к интернету", "error");
    }
  } finally {
    serverLoading = false;
    statusInFlight = false;
  }
}

async function loadServerPing() {
  if (pingInFlight) return;
  pingInFlight = true;
  try {
    const result = await invoke<ServerPingResponse>("get_server_ping");
    serverPing = result.ping || 0;
  } catch (e) {
    handleError(e, "loadServerPing");
    serverPing = 0;
  } finally {
    pingInFlight = false;
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
    get reachability() { return serverReachability; },
    get hasStats() { return serverHasStats; },
    get notifications() { return notifications; },
    addNotification,
    loadServerStatus,
    loadServerPing,
    refreshServerStatus,
  };
}
