/**
 * ASTRA Launcher — Configuration Constants
 * Centralized configuration for easy maintenance
 */

// Server Configuration (used by Rust backend, kept for reference)
// export const SERVER_ADDRESS = "185.176.94.21:30120";

// Update Configuration
export const UPDATE_CHECK_INTERVAL = 3600000; // 1 hour
export const FIVEM_DOWNLOAD_URL = "https://runtime.fivem.net/client/FiveM.exe";

// Font Configuration
export const FONTS = {
  primary: "'Proxima Nova', sans-serif",
  primaryBold: "'Proxima Nova Bold', sans-serif",
  primarySemibold: "'Proxima Nova Semibold', sans-serif",
  logo: "'Armor Piercing 2.0 BB', 'Impact', sans-serif",
};

// Color Configuration
export const COLORS = {
  primary: "#f64a46", // Red accent
  background: "#0d0d0d", // Main bg
  sidebarBg: "#1b1b1b", // Sidebar bg
  success: "#15ff00", // Green success
  textPrimary: "#ffffff",
  textSecondary: "#ffffff/50",
  textTertiary: "#ffffff/30",
};

// Size Configuration
export const SIZES = {
  sidebarWidth: 157, // px
  mainWindow: { width: 960, height: 600 },
  splashWindow: { width: 300, height: 358 },
  // ASTRA Logo sizes (responsive, DPI-scaled)
  logoResponsive: "clamp(32px, 4.2vw, 40px)",
  logoModal: "28px",
};

// Audio Configuration
export const AUDIO = {
  clickFrequency: 800, // Hz
  clickDuration: 100, // ms
  hoverFrequency: 600, // Hz
  hoverDuration: 50, // ms
  musicDefaultVolume: 0.3,
  musicStreamUrl: "https://streams.fluxfm.de/chillhop",
};

// Timing Configuration
export const TIMING = {
  animationDuration: 300, // ms
  transitionDuration: 700, // ms
  serverStatusInterval: 10000, // 10s — обновление статуса сервера
  serverPingInterval: 10000, // 10s — обновление пинга
  updateCheckInterval: 1800000, // 30 мин — проверка обновлений
  notificationTimeout: 4000, // 4s — таймаут уведомлений
  statusMessageTimeout: 3000, // 3s — таймаут статус-сообщений
  installerPollInterval: 3000, // 3s — интервал проверки установщика
  installerPollTimeout: 600000, // 10 мин — таймаут ожидания установщика
  modalAnimationDuration: 150, // ms
  launcherReadyDelay: 300, // ms — задержка перед показом UI
};

// Keyboard Keys
export const KEYS = {
  ESCAPE: "Escape",
  ENTER: "Enter",
};

// Dialog Configuration
export const DIALOG = {
  backdrop: {
    class:
      "absolute inset-0 z-[300] flex items-center justify-center bg-black/70 backdrop-blur-sm animate-fade-in",
  },
  content: {
    class:
      "bg-[#1b1b1b] border border-white/10 rounded-xl px-8 py-6 max-w-md w-full shadow-2xl",
  },
};

// Utility: Format file size
export function fmtSize(b: number): string {
  if (b === 0) return "0 Б";
  const k = 1024;
  const s = ["Б", "КБ", "МБ", "ГБ"];
  const i = Math.floor(Math.log(b) / Math.log(k));
  return (b / Math.pow(k, i)).toFixed(1) + " " + s[i];
}
