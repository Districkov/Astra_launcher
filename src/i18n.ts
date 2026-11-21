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
  "play.rageNotFound": "RAGE:MP не найден. Укажите путь в настройках.",
  "play.rageLaunched": "GTA V запущен ✓",
  "play.downloadRage": "Установить RAGE:MP",
  "play.checkAgain": "Проверить",
  "play.installRage": "Установить RAGE:MP",
  "play.serverUnreachable": "Сервер недоступен",
  "play.noStats": "Статистика сервера недоступна — запуск всё равно возможен",

  // Launch phases
  "launch.checkClient": "Проверка клиента",
  "launch.connectServer": "Подключение к серверу",
  "launch.loadResources": "Загрузка ресурсов",
  "launch.startRage": "Запуск GTA V",

  // Settings
  "settings.title": "Настройки",
  "settings.ragePath": "Путь к RAGE:MP",
  "settings.ragePathHint": "Укажите папку с GTA5.exe и RageMP.exe, если лаунчер не нашёл её автоматически.",
  "settings.select": "Выбрать",
  "settings.autoFind": "Найти автоматически",
  "settings.rageFound": "✓ Клиент найден",
  "settings.rageNotFound": "✗ Клиент не найден — выберите папку вручную",
  "settings.aboutLauncher": "О лаунчере",
  "settings.serverAddress": "Сервер:",
  "settings.serverAddressLegacy": "Сервер: 89.35.130.191:22005",
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
  "onboarding.searchRage": "Поиск RAGE:MP",
  "onboarding.searching": "Ищем RAGE:MP на вашем компьютере…",
  "onboarding.found": "✓ RAGE:MP найден автоматически",
  "onboarding.notFound": "RAGE:MP не найден на вашем компьютере",
  "onboarding.next": "Далее",
  "onboarding.selectPath": "Указать папку вручную",
  "onboarding.downloadRage": "Установить RAGE:MP",
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

  // Patches
  "patches.title": "Патчи updater.exe",
  "patches.applied": "{0}/{1} патчей применено",
  "patches.applyOnPlay": "Патчи применяются при нажатии «Играть»; кнопка выше только проверяет состояние.",
  "patches.source": "Источник:",
  "patches.sourceBuiltin": "встроенный набор",
  "patches.sourceServer": "сервер",
  "patches.referenceWarn": "⚠ Сборка updater.exe отличается — патчи будут найдены по сигнатурам",
  "patches.upstream": "Таблица патчей портирована из {0} (MIT, открытый код)",
  "patches.updaterMissing": "updater.exe не найден — укажите путь к RAGE:MP",
  "patches.refresh": "Обновить состояние патчей",

  // Whitelist bypass
  "whitelist.title": "Обход белого списка доменов",
  "whitelist.hint": "ragemp_v.exe принимает только *.gta5rp.com и *.grandrp.com. Два способа обойти это ограничение.",
  "whitelist.modeRaw": "Прямой IP",
  "whitelist.modeHosts": "Домен + hosts",
  "whitelist.rawHint": "Автоподключение по «сырому» IP через rui\\index.rui — белый список фильтрует только домены.",
  "whitelist.rawHint2": "Не требует прав администратора. index.rui резервируется перед изменением.",
  "whitelist.hostsHint": "Домен подменяется через файл hosts. Требуются права администратора (запрос UAC).",
  "whitelist.mappingFound": "✓ {0} → {1}",
  "whitelist.mappingMissing": "Маппинг в hosts не найден",
  "whitelist.apply": "Применить обход",
  "whitelist.applying": "Применение…",
  "whitelist.remove": "Снять обход",
  "whitelist.removing": "Снятие…",
  "whitelist.refresh": "Обновить",
  "whitelist.registry": "Реестр:",
  "whitelist.denied": "Обход не применён (нужны права администратора)",
  "whitelist.restoreRui": "Восстановить index.rui из бэкапа",

  // RUI editor
  "rui.title": "Редактор интерфейса (index.rui)",
  "rui.hint": "Правка встроенного интерфейса RAGE:MP. Архив резервируется перед изменением.",
  "rui.state": "Архив: {0} файлов",
  "rui.injected": "Автоподключение внедрено",
  "rui.notInjected": "Автоподключение не внедрено",
  "rui.backupExists": "Бэкап есть",
  "rui.noBackup": "Бэкапа нет",
  "rui.unparsable": "Формат архива не распознан",
  "rui.windowTitle": "Заголовок окна",
  "rui.windowTitlePlaceholder": "Например: ASTRA RP",
  "rui.autoConnect": "Автоподключение при запуске",
  "rui.autoConnectIp": "IP сервера",
  "rui.autoConnectPort": "Порт",
  "rui.autoConnectDelay": "Задержка, мс",
  "rui.serverListOverride": "Подменить список серверов",
  "rui.apply": "Применить",
  "rui.preview": "Проверить",
  "rui.restore": "Восстановить из бэкапа",
  "rui.applied": "Применено: {0} файл(ов) изменено",
  "rui.noChanges": "Изменений нет",
  "rui.previewTitle": "Будет изменено:",
  "rui.warnings": "Предупреждения:",
  "rui.restored": "index.rui восстановлен из бэкапа ✓",
  "rui.noBackupToRestore": "Бэкап index.rui не найден",

  // Launch path
  "serverAddr.title": "Адрес сервера",
  "serverAddr.hint": "IP и порт сервера. Для RAGE:MP порт обычно 22005 (30120 — нестандартный порт другого клиента).",
  "serverAddr.save": "Сохранить",

  "launchPath.title": "Путь запуска",
  "launchPath.hint": "Через что лаунчер запускает игру.",
  "launchPath.updater": "Через updater.exe",
  "launchPath.updaterHint": "Официальный путь. updater проверяет CDN — патчи обходят проверку.",
  "launchPath.client": "Напрямую ragemp_v.exe",
  "launchPath.clientHint": "Пропускает updater и проверку CDN целиком.",
  "launchPath.clientWarning": "Путь не проверен на вашей сборке. Если игра не запустится — вернитесь на официальный путь.",
  "launchPath.diag": "Диагностика",
  "launchPath.diagTarget": "Целевой бинарник",
  "launchPath.diagPresent": "найден",
  "launchPath.diagMissing": "не найден",
  "launchPath.diagRegistry": "Реестр",
  "launchPath.diagRegistryOk": "оба набора ключей записаны",
  "launchPath.diagRegistryPartial": "записан только один набор",
  "launchPath.diagMatches": "совпадает с сервером",
  "launchPath.diagPending": "будет записан при запуске",
  "launchPath.diagMismatch": "не совпадает с сервером",
  "launchPath.diagRui": "Автоподключение",
  "launchPath.diagRuiIn": "внедрено",
  "launchPath.diagRuiPending": "будет внедрено при нажатии «Играть»",
  "launchPath.diagRuiOut": "не внедрено после запуска",
  "launchPath.ruiToggle": "Внедрять автоподключение в index.rui",
  "launchPath.ruiToggleHint": "Экспериментально: любая правка index.rui заставляет клиент этой сборки показывать чёрное окно «404 error: File not found». Включено — меню выбора сервера пропадёт, но клиент может не запуститься. Выключено — всё работает штатно, сервер выбирается одним кликом в меню.",
  "launchPath.refresh": "Обновить",
  "launchPath.clearRegistry": "Очистить реестр",
  "launchPath.notes": "Замечания:",
  "launchPath.noNotes": "Замечаний нет",

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
  "play.rageNotFound": "RAGE:MP not found. Set the path in settings.",
  "play.rageLaunched": "GTA V launched ✓",
  "play.downloadRage": "Install RAGE:MP",
  "play.checkAgain": "Check again",
  "play.installRage": "Install RAGE:MP",
  "play.serverUnreachable": "Server unreachable",
  "play.noStats": "Server stats unavailable — you can still launch",

  // Launch phases
  "launch.checkClient": "Checking client",
  "launch.connectServer": "Connecting to server",
  "launch.loadResources": "Loading resources",
  "launch.startRage": "Starting GTA V",

  // Settings
  "settings.title": "Settings",
  "settings.ragePath": "RAGE:MP Path",
  "settings.ragePathHint": "Specify the folder with GTA5.exe and RageMP.exe if the launcher didn't find it automatically.",
  "settings.select": "Select",
  "settings.autoFind": "Find automatically",
  "settings.rageFound": "✓ Client found",
  "settings.rageNotFound": "✗ Client not found — select folder manually",
  "settings.aboutLauncher": "About Launcher",
  "settings.serverAddress": "Server:",
  "settings.serverAddressLegacy": "Server: 89.35.130.191:22005",
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
  "onboarding.searchRage": "Finding RAGE:MP",
  "onboarding.searching": "Searching for RAGE:MP on your computer…",
  "onboarding.found": "✓ RAGE:MP found automatically",
  "onboarding.notFound": "RAGE:MP not found on your computer",
  "onboarding.next": "Next",
  "onboarding.selectPath": "Select folder manually",
  "onboarding.downloadRage": "Install RAGE:MP",
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

  // Patches
  "patches.title": "updater.exe patches",
  "patches.applied": "{0}/{1} patches applied",
  "patches.applyOnPlay": "Patches are applied when you press Play; the button above only checks their status.",
  "patches.source": "Source:",
  "patches.sourceBuiltin": "built-in table",
  "patches.sourceServer": "server",
  "patches.referenceWarn": "⚠ updater.exe build differs — patches will be located by signature",
  "patches.upstream": "Patch table ported from {0} (MIT, open source)",
  "patches.updaterMissing": "updater.exe not found — set the RAGE:MP path",
  "patches.refresh": "Refresh patch state",

  // Whitelist bypass
  "whitelist.title": "Domain whitelist bypass",
  "whitelist.hint": "ragemp_v.exe only accepts *.gta5rp.com and *.grandrp.com. Two ways around it.",
  "whitelist.modeRaw": "Raw IP",
  "whitelist.modeHosts": "Domain + hosts",
  "whitelist.rawHint": "Auto-connect by raw IP via rui\\index.rui — the whitelist only filters domains.",
  "whitelist.rawHint2": "No administrator rights needed. index.rui is backed up before editing.",
  "whitelist.hostsHint": "The domain is mapped through the hosts file. Requires administrator rights (UAC).",
  "whitelist.mappingFound": "✓ {0} → {1}",
  "whitelist.mappingMissing": "No hosts mapping found",
  "whitelist.apply": "Apply bypass",
  "whitelist.applying": "Applying…",
  "whitelist.remove": "Remove bypass",
  "whitelist.removing": "Removing…",
  "whitelist.refresh": "Refresh",
  "whitelist.registry": "Registry:",
  "whitelist.denied": "Bypass not applied (administrator rights required)",
  "whitelist.restoreRui": "Restore index.rui from backup",

  // RUI editor
  "rui.title": "Interface editor (index.rui)",
  "rui.hint": "Edit the embedded RAGE:MP interface. The archive is backed up before any change.",
  "rui.state": "Archive: {0} files",
  "rui.injected": "Auto-connect injected",
  "rui.notInjected": "Auto-connect not injected",
  "rui.backupExists": "Backup present",
  "rui.noBackup": "No backup",
  "rui.unparsable": "Archive format not recognised",
  "rui.windowTitle": "Window title",
  "rui.windowTitlePlaceholder": "e.g. ASTRA RP",
  "rui.autoConnect": "Auto-connect on launch",
  "rui.autoConnectIp": "Server IP",
  "rui.autoConnectPort": "Port",
  "rui.autoConnectDelay": "Delay, ms",
  "rui.serverListOverride": "Override the server list",
  "rui.apply": "Apply",
  "rui.preview": "Preview",
  "rui.restore": "Restore from backup",
  "rui.applied": "Applied: {0} file(s) changed",
  "rui.noChanges": "No changes",
  "rui.previewTitle": "Will change:",
  "rui.warnings": "Warnings:",
  "rui.restored": "index.rui restored from backup ✓",
  "rui.noBackupToRestore": "No index.rui backup found",

  // Launch path
  "serverAddr.title": "Server address",
  "serverAddr.hint": "Server IP and port. For RAGE:MP the port is usually 22005 (30120 is a non-standard port of another client).",
  "serverAddr.save": "Save",

  "launchPath.title": "Launch path",
  "launchPath.hint": "Which binary the launcher hands control to.",
  "launchPath.updater": "Via updater.exe",
  "launchPath.updaterHint": "Official path. The updater checks the CDN — our patches bypass it.",
  "launchPath.client": "Directly via ragemp_v.exe",
  "launchPath.clientHint": "Skips the updater and the CDN check entirely.",
  "launchPath.clientWarning": "Unverified on your build. If the game does not start, switch back to the official path.",
  "launchPath.diag": "Diagnostics",
  "launchPath.diagTarget": "Target binary",
  "launchPath.diagPresent": "found",
  "launchPath.diagMissing": "missing",
  "launchPath.diagRegistry": "Registry",
  "launchPath.diagRegistryOk": "both key families written",
  "launchPath.diagRegistryPartial": "only one key family written",
  "launchPath.diagMatches": "matches the server",
  "launchPath.diagPending": "will be written on launch",
  "launchPath.diagMismatch": "does not match the server",
  "launchPath.diagRui": "Auto-connect",
  "launchPath.diagRuiIn": "injected",
  "launchPath.diagRuiPending": "will be injected when you press Play",
  "launchPath.diagRuiOut": "not injected after launch",
  "launchPath.ruiToggle": "Inject auto-connect into index.rui",
  "launchPath.ruiToggleHint": "Experimental: any edit to index.rui makes this client build show a black \"404 error: File not found\" window. On — the server menu is skipped but the client may not start. Off — everything works normally and the server is one click away in the menu.",
  "launchPath.refresh": "Refresh",
  "launchPath.clearRegistry": "Clear registry",
  "launchPath.notes": "Notes:",
  "launchPath.noNotes": "No issues found",

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
