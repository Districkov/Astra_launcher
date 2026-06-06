/**
 * ASTRA Launcher — i18n (Internationalization) System
 * Simple key-based translation with Russian as default
 */

type TranslationKey = keyof typeof ru;
type Translations = Record<string, string>;

const ru: Translations = {
  // Sidebar
  "menu.play": "Играть",
  "menu.store": "Магазин",
  "menu.news": "Новости",
  "menu.forum": "Форум",
  "menu.discord": "Discord",
  "menu.settings": "Настройки",
  "sidebar.online": "Онлайн:",
  "sidebar.offline": "Офлайн",

  // Play section
  "play.button": "Играть",
  "play.launching": "Запуск…",
  "play.cacheDownloading": "Скачивание кеша",
  "play.cacheExtracting": "Распаковка кеша",
  "play.fivemNotFound": "FiveM не найден. Укажите путь в настройках.",
  "play.fivemLaunched": "FiveM запущен ✓",
  "play.downloadFivem": "Скачать FiveM",
  "play.checkAgain": "Проверить",
  "play.installFivem": "Установить FiveM",

  // Launch phases
  "launch.checkClient": "Проверка клиента",
  "launch.connectServer": "Подключение к серверу",
  "launch.loadResources": "Загрузка ресурсов",
  "launch.startFivem": "Запуск FiveM",

  // Settings
  "settings.title": "Настройки",
  "settings.fivemPath": "Путь к FiveM",
  "settings.fivemPathHint": "Укажите расположение FiveM.exe, если лаунчер не нашёл его автоматически.",
  "settings.select": "Выбрать",
  "settings.autoFind": "Найти автоматически",
  "settings.fivemFound": "✓ FiveM.exe найден",
  "settings.fivemNotFound": "✗ FiveM.exe не найден — выберите вручную",
  "settings.aboutLauncher": "О лаунчере",
  "settings.serverAddress": "Сервер: 185.176.94.21:30120",
  "settings.serverOnline": "Сервер онлайн",
  "settings.serverOffline": "Сервер офлайн",
  "settings.refreshStatus": "Обновить статус",
  "settings.refreshing": "Обновление…",
  "settings.updateAvailable": "Доступно обновление до",
  "settings.updateLatest": "✓ Установлена последняя версия",
  "settings.checkUpdates": "Проверить обновления",
  "settings.checking": "Проверка…",
  "settings.downloadInstall": "Скачать и установить",

  // Update modal
  "update.title": "Обновление доступно",
  "update.current": "Текущая версия:",
  "update.latest": "Новая версия:",
  "update.download": "Скачать и установить",
  "update.downloading": "Скачивание…",
  "update.installed": "Обновление установлено! Перезапуск…",

  // Notifications
  "notif.serverOnline": "🟢 Сервер снова онлайн!",
  "notif.serverOffline": "🔴 Сервер перешёл в офлайн",
  "notif.noConnection": "📡 Нет подключения к интернету",
  "notif.updateAvailable": "🔄 Доступно обновление:",
  "notif.updateInstalled": "✅ Обновление установлено! Перезапуск...",
  "notif.precacheFailed": "⚠️ Предзагрузка не удалась:",
  "notif.loggedOut": "Вы вышли из аккаунта",

  // Onboarding
  "onboarding.welcome": "Добро пожаловать!",
  "onboarding.welcomeDesc": "Лаунчер для подключения к ASTRA RP. Настроим всё за пару шагов.",
  "onboarding.start": "Начать",
  "onboarding.searchFivem": "Поиск FiveM",
  "onboarding.searching": "Ищем FiveM на вашем компьютере…",
  "onboarding.found": "✓ FiveM найден автоматически",
  "onboarding.notFound": "FiveM не найден на вашем компьютере",
  "onboarding.next": "Далее",
  "onboarding.selectPath": "Выбрать вручную",
  "onboarding.downloadFivem": "Скачать FiveM",
  "onboarding.nickname": "Введите никнейм",
  "onboarding.nicknameHint": "Это имя будет отображаться в игре",
  "onboarding.nicknamePlaceholder": "Ваш никнейм",
  "onboarding.saving": "Сохранение…",
  "onboarding.ready": "Всё готово!",
  "onboarding.readyDesc": "Лаунчер настроен и готов к работе. Нажмите «Играть» для подключения!",
  "onboarding.finish": "Начать игру",

  // Modals
  "modal.closeTitle": "Закрыть лаунчер?",
  "modal.closeDesc": "Лаунчер будет закрыт. Вы можете открыть его снова из системного трея.",
  "modal.close": "Закрыть",
  "modal.cancel": "Отмена",
  "modal.minimize": "Свернуть в трей",
  "modal.logoutTitle": "Выйти из аккаунта?",
  "modal.logoutDesc": "Вы будете отключены от профиля.",
  "modal.logout": "Выйти",

  // Misc
  "misc.notFound": "Не найден",
  "misc.sectionInDev": "Раздел в разработке",
  "misc.section": "Раздел",
};

const en: Translations = {
  // Sidebar
  "menu.play": "Play",
  "menu.store": "Store",
  "menu.news": "News",
  "menu.forum": "Forum",
  "menu.discord": "Discord",
  "menu.settings": "Settings",
  "sidebar.online": "Online:",
  "sidebar.offline": "Offline",

  // Play section
  "play.button": "Play",
  "play.launching": "Launching…",
  "play.cacheDownloading": "Downloading cache",
  "play.cacheExtracting": "Extracting cache",
  "play.fivemNotFound": "FiveM not found. Set the path in settings.",
  "play.fivemLaunched": "FiveM launched ✓",
  "play.downloadFivem": "Download FiveM",
  "play.checkAgain": "Check again",
  "play.installFivem": "Install FiveM",

  // Launch phases
  "launch.checkClient": "Checking client",
  "launch.connectServer": "Connecting to server",
  "launch.loadResources": "Loading resources",
  "launch.startFivem": "Starting FiveM",

  // Settings
  "settings.title": "Settings",
  "settings.fivemPath": "FiveM Path",
  "settings.fivemPathHint": "Specify the location of FiveM.exe if the launcher didn't find it automatically.",
  "settings.select": "Select",
  "settings.autoFind": "Find automatically",
  "settings.fivemFound": "✓ FiveM.exe found",
  "settings.fivemNotFound": "✗ FiveM.exe not found — select manually",
  "settings.aboutLauncher": "About Launcher",
  "settings.serverAddress": "Server: 185.176.94.21:30120",
  "settings.serverOnline": "Server online",
  "settings.serverOffline": "Server offline",
  "settings.refreshStatus": "Refresh status",
  "settings.refreshing": "Refreshing…",
  "settings.updateAvailable": "Update available to",
  "settings.updateLatest": "✓ Latest version installed",
  "settings.checkUpdates": "Check for updates",
  "settings.checking": "Checking…",
  "settings.downloadInstall": "Download and install",

  // Update modal
  "update.title": "Update Available",
  "update.current": "Current version:",
  "update.latest": "New version:",
  "update.download": "Download and install",
  "update.downloading": "Downloading…",
  "update.installed": "Update installed! Restarting…",

  // Notifications
  "notif.serverOnline": "🟢 Server is back online!",
  "notif.serverOffline": "🔴 Server went offline",
  "notif.noConnection": "📡 No internet connection",
  "notif.updateAvailable": "🔄 Update available:",
  "notif.updateInstalled": "✅ Update installed! Restarting...",
  "notif.precacheFailed": "⚠️ Precache failed:",
  "notif.loggedOut": "You have been logged out",

  // Onboarding
  "onboarding.welcome": "Welcome!",
  "onboarding.welcomeDesc": "Launcher for connecting to ASTRA RP. Let's set everything up in a few steps.",
  "onboarding.start": "Get Started",
  "onboarding.searchFivem": "Finding FiveM",
  "onboarding.searching": "Searching for FiveM on your computer…",
  "onboarding.found": "✓ FiveM found automatically",
  "onboarding.notFound": "FiveM not found on your computer",
  "onboarding.next": "Next",
  "onboarding.selectPath": "Select manually",
  "onboarding.downloadFivem": "Download FiveM",
  "onboarding.nickname": "Enter your nickname",
  "onboarding.nicknameHint": "This name will be displayed in-game",
  "onboarding.nicknamePlaceholder": "Your nickname",
  "onboarding.saving": "Saving…",
  "onboarding.ready": "All set!",
  "onboarding.readyDesc": "The launcher is configured and ready. Press \"Play\" to connect!",
  "onboarding.finish": "Start Playing",

  // Modals
  "modal.closeTitle": "Close Launcher?",
  "modal.closeDesc": "The launcher will be closed. You can reopen it from the system tray.",
  "modal.close": "Close",
  "modal.cancel": "Cancel",
  "modal.minimize": "Minimize to tray",
  "modal.logoutTitle": "Log out?",
  "modal.logoutDesc": "You will be disconnected from your profile.",
  "modal.logout": "Log out",

  // Misc
  "misc.notFound": "Not found",
  "misc.sectionInDev": "Section in development",
  "misc.section": "Section",
};

type Locale = "ru" | "en";

const translations: Record<Locale, Translations> = { ru, en };

let currentLocale: Locale = "ru";

/**
 * Get the current locale
 */
export function getLocale(): Locale {
  return currentLocale;
}

/**
 * Set the current locale
 */
export function setLocale(locale: Locale): void {
  if (translations[locale]) {
    currentLocale = locale;
  }
}

/**
 * Get a translated string by key
 * Falls back to Russian if key not found in current locale
 */
export function t(key: string): string {
  return translations[currentLocale]?.[key] ?? translations.ru?.[key] ?? key;
}

/**
 * Get a translated string with interpolation
 * Example: t("notif.updateAvailable") + " v1.2.0"
 */
export function tf(key: string, ...args: string[]): string {
  let result = t(key);
  args.forEach((arg, i) => {
    result = result.replace(`{${i}}`, arg);
  });
  return result;
}
