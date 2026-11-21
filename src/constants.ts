/**
 * ASTRA Launcher — Configuration Constants
 * Centralized configuration for easy maintenance
 */

// Font Configuration
export const FONTS = {
  primary: "'Proxima Nova', sans-serif",
  primaryBold: "'Proxima Nova Bold', sans-serif",
  primarySemibold: "'Proxima Nova Semibold', sans-serif",
  logo: "'Armor Piercing 2.0 BB', 'Impact', sans-serif",
};

// Color Configuration
export const COLORS = {
  primary: "#f64a46",
  background: "#0d0d0d",
  sidebarBg: "#1b1b1b",
  success: "#15ff00",
  textPrimary: "#ffffff",
  textSecondary: "#ffffff/50",
  textTertiary: "#ffffff/30",
};

// Size Configuration
export const SIZES = {
  sidebarWidth: 157,
  mainWindow: { width: 960, height: 600 },
  splashWindow: { width: 300, height: 358 },
  logoResponsive: "clamp(32px, 4.2vw, 40px)",
  logoModal: "28px",
};

// Audio Configuration
export const AUDIO = {
  clickFrequency: 800,
  clickDuration: 100,
  hoverFrequency: 600,
  hoverDuration: 50,
  musicDefaultVolume: 0.3,
  musicStreamUrl: "https://streams.fluxfm.de/chillhop",
};

// Timing Configuration
export const TIMING = {
  animationDuration: 300,
  transitionDuration: 700,
  serverStatusInterval: 10000,
  serverPingInterval: 10000,
  updateCheckInterval: 1800000,
  notificationTimeout: 4000,
  statusMessageTimeout: 3000,
  modalAnimationDuration: 150,
  launcherReadyDelay: 300,
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
