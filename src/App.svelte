<!--
  ASTRA Launcher — Frontend (Svelte 5 + Tailwind CSS)
  ==================================================
  Дизайн на основе imdex.tsx — тёмная тема с акцентами #f64a46

  Структура:
  ┌──────────┬──────────────────────────┐
  │ Sidebar  │       Main Area          │
  │ #1b1b1b  │   GTA V bg + ASTRA       │
  │          │   [Кнопка «Играть»]      │
  │ Logo     │   [Настройки]            │
  │ Status   │                          │
  │ Menu     │                          │
  │ Profile  │                          │
  └──────────┴──────────────────────────┘
-->

<script>
  import { invoke } from "@tauri-apps/api/core";
  import { getVersion } from "@tauri-apps/api/app";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { open } from "@tauri-apps/plugin-dialog";
  import { open as openUrl } from "@tauri-apps/plugin-shell";
  import { relaunch } from "@tauri-apps/plugin-process";
  import { listen } from "@tauri-apps/api/event";
  import { onMount } from "svelte";
  import { SIZES, COLORS, FONTS } from "./constants";
  import { handleError, ErrorLogger } from "./errorHandling";
  import Background from "./components/Background.svelte";
  import Notifications from "./components/Notifications.svelte";
  import UpdateIndicator from "./components/UpdateIndicator.svelte";
  import WindowControls from "./components/WindowControls.svelte";
  import CloseConfirmModal from "./components/CloseConfirmModal.svelte";
  import LogoutConfirmModal from "./components/LogoutConfirmModal.svelte";
  import UpdateModal from "./components/UpdateModal.svelte";
  import Sidebar from "./components/Sidebar.svelte";
  import Settings from "./components/Settings.svelte";
  import PlaySection from "./components/PlaySection.svelte";
  import OnboardingModal from "./components/OnboardingModal.svelte";

  let appWindow;
  try {
    appWindow = getCurrentWindow();
  } catch (e) {
    // window not available
    appWindow = null;
  }

  // ── Состояние (Svelte 5 Runes) ──────────────────
  let isLaunching = $state(false);
  let statusMessage = $state("");
  let activeMenu = $state("play");
  let previousMenu = $state("play");

  // Анимация загрузки при подключении
  let launchPhase = $state(0); // 0=нет, 1=проверка, 2=подключение, 3=загрузка, 4=запуск
  let launchProgress = $state(0); // 0-100
  const launchPhases = [
    { label: "Проверка клиента", icon: "check" },
    { label: "Подключение к серверу", icon: "connect" },
    { label: "Загрузка ресурсов", icon: "download" },
    { label: "Запуск FiveM", icon: "play" },
  ];

  // Статус сервера (реальный)
  let serverOnline = $state(false);
  let serverPlayers = $state(0);
  let serverMaxPlayers = $state(0);
  let serverLoading = $state(true);
  let prevServerOnline = $state(null); // для отслеживания изменений

  // FiveM путь
  let fivemPath = $state("");
  let fivemFound = $state(false);

  // Скачивание FiveM
  let isDownloading = $state(false);
  let downloadPercent = $state(0);
  let downloadSize = $state("");
  let downloadError = $state("");

  // #5 — Автообновление (tauri-plugin-updater — бесшовное, без установщика)
  let updateAvailable = $state(false);
  let updateInfo = $state({ current_version: "", latest_version: "", release_notes: "" });
  let updateChecked = $state(false);
  let updateDownloading = $state(false);
  let updateDownloadPercent = $state(0);
  let updateDownloaded = $state(0);
  let updateTotal = $state(0);
  let updateError = $state("");
  let showUpdateModal = $state(false);
  let pendingUpdate = $state(null); // объект Update от плагина

  // #7 — Уведомления сервера
  let notifications = $state([]);
  let notificationId = 0;

  // #9 — Имя пользователя
  let username = $state("Player");

  // #11 — Звуки кликов и наведения
  let clickSound = null;
  let hoverSound = null;

  // #2 — Пинг сервера
  let serverPing = $state(0);

  // #14 — Музыка в лаунчере
  // #13 — Discord Rich Presence (флаг)
  let discordRpcActive = $state(false);

  let showCloseConfirm = $state(false);
  let showLogoutConfirm = $state(false);

  // Анимация появления лаунчера
  let launcherReady = $state(false);

  // ── Онбординг (первый запуск) ──────────────────
  let onboardingActive = $state(false);
  let onboardingStep = $state(1); // 1=Приветствие, 2=FiveM, 3=Никнейм, 4=Готово
  let onboardingFivemSearching = $state(false);
  let onboardingFivemResult = $state(null); // null | 'found' | 'not_found'
  let onboardingNickname = $state("");
  let onboardingNicknameSaving = $state(false);

  // Параллакс — смещение фона при движении мыши
  let parallaxX = $state(0);
  let parallaxY = $state(0);

  // DPI масштаб для ASTRA текста на 2K/4K
  let dpiScale = $state(1);

  function handleMouseMove(e) {
    const cx = window.innerWidth / 2;
    const cy = window.innerHeight / 2;
    parallaxX = ((e.clientX - cx) / cx) * 12;
    parallaxY = ((e.clientY - cy) / cy) * 8;
  }

  // Частицы — позиции кэшируются один раз
  let particleStyles = $state([]);
  function initParticles() {
    particleStyles = Array.from({ length: 30 }, () => ({
      left: `${Math.random() * 100}%`,
      top: `${Math.random() * 100}%`,
      delay: `${Math.random() * 8}s`,
      duration: `${4 + Math.random() * 6}s`,
      width: `${1 + Math.random() * 2}px`,
      height: `${1 + Math.random() * 2}px`,
    }));
  }



  // Производное: текст кнопки
  let playLabel = $derived(
    precacheDownloading ? `Скачивание кеша ${precachePercent}%${precacheTotal ? ' (' + precacheDownloaded + '/' + precacheTotal + ')' : ''}` :
    precacheExtracting ? `Распаковка кеша ${precachePercent}%` :
    launchPhase > 0 ? launchPhases[launchPhase - 1]?.label || "Запуск…" :
    isLaunching ? "Запуск…" : "Играть"
  );

  // Пункты бокового меню (из imdex.tsx)
  const menuItems = [
    { id: "play",       label: "Играть",      icon: "play",       active: true,  group: 1 },
    { id: "store",      label: "Магазин",     icon: "store",      active: false, group: 1 },
    { id: "news",       label: "Новости",     icon: "news",       active: false, group: 2 },
    { id: "forum",      label: "Форум",       icon: "forum",      active: false, group: 2 },
    { id: "discord",    label: "Discord",      icon: "discord",    active: false, group: 3 },
    { id: "settings",   label: "Настройки",   icon: "settings",   active: false, group: 3 },
  ];

  // ── #11 Звуки ──────────────────────────────────
  function playClickSound() {
    if (!clickSound) return;
    try {
      clickSound.play();
    } catch (e) {}
  }

  function playHoverSound() {
    if (!hoverSound) return;
    try {
      hoverSound.play();
    } catch (e) {}
  }

  // ── #7 Уведомления ──────────────────────────────
  function addNotification(message, type = "info") {
    const id = ++notificationId;
    notifications = [...notifications, { id, message, type }];
    setTimeout(() => {
      notifications = notifications.filter(n => n.id !== id);
    }, 4000);
  }

  // ── Статус сервера ──────────────────────────────
  let serverInitialized = $state(false); // первый запуск загружен?

  async function loadServerStatus() {
    // Показываем «Загрузка…» только при первом запуске
    if (!serverInitialized) serverLoading = true;
    try {
      const status = await invoke("get_server_status");
      const wasOnline = prevServerOnline;
      serverOnline = status.online;
      serverPlayers = status.players;
      serverMaxPlayers = status.max_players;
      prevServerOnline = status.online;
      serverInitialized = true;

      // #7 — Уведомление при изменении статуса
      if (wasOnline !== null && wasOnline !== status.online) {
        if (status.online) {
          addNotification("🟢 Сервер снова онлайн!", "success");
        } else {
          addNotification("🔴 Сервер перешёл в офлайн", "error");
        }
      }
    } catch (e) {
      serverOnline = false;
    } finally {
      serverLoading = false;
    }
  }

  // Обновить статус сервера + пинг (по кнопке)
  async function refreshServerStatus() {
    playClickSound();
    serverLoading = true;
    await Promise.all([loadServerStatus(), loadServerPing()]);
  }

  // ── #5 Проверка обновлений (tauri-plugin-updater) ──────
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

  async function startUpdateDownload() {
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
      await pendingUpdate.downloadAndInstall((event) => {
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
      // downloadAndInstall скачал, проверил подпись и запустил установщик NSIS
      // Установщик обновит приложение — перезапускаем текущий процесс
      addNotification("✅ Обновление установлено! Перезапуск...", "success");
      setTimeout(() => {
        relaunch();
      }, 1000);
    } catch (e) {
      updateError = String(e);
      updateDownloading = false;
    }
  }

  async function installUpdate() {
    playClickSound();
    if (!pendingUpdate) return;
    try {
      // Если обновление уже скачано через download() — устанавливаем и перезапускаем
      await pendingUpdate.install();
      await relaunch();
    } catch (e) {
      updateError = String(e);
    }
  }

  function openUpdateModal() {
    playClickSound();
    showUpdateModal = true;
  }

  function closeUpdateModal() {
    playClickSound();
    showUpdateModal = false;
  }

  // ── #9 Имя пользователя ────────────────────────
  async function loadUsername() {
    try {
      const name = await invoke("get_username");
      username = name || "Player";
    } catch (e) {
      username = "Player";
    }
  }

  // ── #2 Пинг сервера ────────────────────────────
  async function loadServerPing() {
    try {
      const result = await invoke("get_server_ping");
      serverPing = result.ping || 0;
    } catch (e) {
      serverPing = 0;
    }
  }

  // ── #13 Discord Rich Presence ──────────────────
  async function initDiscordRpc() {
    try {
      const ok = await invoke("set_discord_rpc", {
        state: "В лаунчере",
        details: "ASTRA RP",
        largeText: "ASTRA Launcher"
      });
      discordRpcActive = ok;
    } catch (e) {
      discordRpcActive = false;
    }
  }

  async function setDiscordRpcPlaying() {
    try {
      await invoke("set_discord_rpc", {
        state: "Играет на сервере",
        details: "ASTRA RP",
        largeText: "ASTRA RP"
      });
    } catch (e) {}
  }

  // ── Путь к FiveM ────────────────────────────────
  async function loadFivemPath() {
    try {
      const path = await invoke("get_fivem_path");
      fivemPath = path;
      fivemFound = !!path;
    } catch (e) {
      fivemFound = false;
    }
  }

  async function autoFindFivem() {
    playClickSound();
    try {
      const path = await invoke("auto_find_fivem");
      if (path) {
        fivemPath = path;
        fivemFound = true;
        statusMessage = "FiveM найден автоматически ✓";
        setTimeout(() => { statusMessage = ""; }, 3000);
      } else {
        statusMessage = "FiveM не найден автоматически";
        setTimeout(() => { statusMessage = ""; }, 3000);
      }
    } catch (e) {
      statusMessage = "Автопоиск недоступен";
      setTimeout(() => { statusMessage = ""; }, 3000);
    }
  }

  async function selectFivemPath() {
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
        setTimeout(() => { statusMessage = ""; }, 2000);
      }
    } catch (e) {
      // dialog error
    }
  }

  // ── Предзагрузка кеша ────────────────────────────
  let precacheNeeded = $state(false);
  let precacheDownloading = $state(false);
  let precacheExtracting = $state(false);
  let precachePercent = $state(0);
  let precacheDownloaded = $state("");
  let precacheTotal = $state("");

  // Форматирование размера файла
  function fmtSize(b) {
    if (b === 0) return "0 Б";
    const k = 1024;
    const s = ["Б", "КБ", "МБ", "ГБ"];
    const i = Math.floor(Math.log(b) / Math.log(k));
    return (b / Math.pow(k, i)).toFixed(1) + " " + s[i];
  }

  // ── Запуск игры ─────────────────────────────────
  async function handlePlay() {
    playClickSound();
    if (isLaunching) return;
    isLaunching = true;
    launchPhase = 1;
    launchProgress = 0;

    // Этап 1: Проверка клиента + предзагрузка кеша (0-50%)
    launchProgress = 5;
    await sleep(300);

    // Проверяем нужна ли предзагрузка
    try {
      const check = await invoke("check_precache_needed");
      precacheNeeded = check.needed;

      if (check.needed) {
        // Предзагрузка кеша — скачивание + распаковка
        launchPhase = 3; // "Загрузка ресурсов"
        precacheDownloading = true;
        precachePercent = 0;

        const unlisten = await listen("precache-progress", (event) => {
          const { phase, downloaded, total, percent, extracted, total_files } = event.payload;

          if (phase === "download") {
            precacheDownloading = true;
            precacheExtracting = false;
            precachePercent = percent;
            precacheDownloaded = fmtSize(downloaded);
            precacheTotal = total > 0 ? fmtSize(total) : "";
            // Прогресс: 5-40% (скачивание)
            launchProgress = 5 + Math.round(percent * 0.35);
          } else if (phase === "extract") {
            precacheDownloading = false;
            precacheExtracting = true;
            precachePercent = percent;
            // Прогресс: 40-50% (распаковка)
            launchProgress = 40 + Math.round(percent * 0.1);
          }
        });

        try {
          await invoke("precache_server_files");
          precacheNeeded = false;
        } catch (error) {
          // Предзагрузка не удалась — не критично, продолжаем
          // precache failed
          addNotification("⚠️ Предзагрузка не удалась: " + error, "error");
        }

        unlisten();
        precacheDownloading = false;
        precacheExtracting = false;
      }

      launchProgress = 50;
    } catch (e) {
      // cache check failed
      launchProgress = 25;
    }

    // Этап 2: Подключение к серверу (50-70%)
    launchPhase = 2;
    launchProgress = 55;
    try {
      const result = await invoke("launch_game");
      launchProgress = 70;
    } catch (error) {
      launchPhase = 0;
      launchProgress = 0;
      isLaunching = false;
      precacheDownloading = false;
      precacheExtracting = false;
      if (error === "FIVEM_NOT_FOUND") {
        statusMessage = "FiveM не найден. Укажите путь в настройках.";
      } else {
        statusMessage = `Ошибка: ${error}`;
      }
      setTimeout(() => { statusMessage = ""; }, 4000);
      return;
    }

    // Этап 3: Загрузка ресурсов (70-90%) — если кеш не предзагружен, FiveM скачает сам
    launchPhase = 3;
    launchProgress = 75;
    await sleep(400);
    launchProgress = 85;
    await sleep(300);

    // Этап 4: Запуск (90-100%)
    launchPhase = 4;
    launchProgress = 95;
    await sleep(300);
    launchProgress = 100;
    await sleep(300);

    // Готово
    launchPhase = 0;
    launchProgress = 0;
    isLaunching = false;
    precacheDownloading = false;
    precacheExtracting = false;
    statusMessage = "FiveM запущен ✓";
    setDiscordRpcPlaying();
    setTimeout(() => { statusMessage = ""; }, 3000);
  }

  function sleep(ms) {
    return new Promise(resolve => setTimeout(resolve, ms));
  }

  // ── Скачивание FiveM ────────────────────────────
  async function downloadAndInstall() {
    playClickSound();
    if (isDownloading) return;
    isDownloading = true;
    downloadPercent = 0;
    downloadSize = "";
    downloadError = "";
    try {
      const unlisten = await listen("download-progress", (event) => {
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
      const poll = setInterval(async () => {
        try {
          const installed = await invoke("check_fivem_installed");
          if (installed) {
            clearInterval(poll);
            await loadFivemPath();
            isDownloading = false;
            statusMessage = "FiveM установлен ✓";
            setTimeout(() => { statusMessage = ""; }, 3000);
          }
        } catch (e) {}
      }, 3000);
    } catch (error) {
      downloadError = `${error}`;
      isDownloading = false;
    }
  }

  async function checkFivemAgain() {
    playClickSound();
    await loadFivemPath();
  }

  // ── Онбординг (первый запуск) ──────────────────
  async function checkOnboarding() {
    try {
      const complete = await invoke("is_onboarding_complete");
      if (!complete) {
        onboardingActive = true;
        onboardingStep = 1;
      }
    } catch (e) {
      // По умолчанию — не показываем
    }
  }

  async function onboardingNext() {
    playClickSound();
    if (onboardingStep === 1) {
      // Переход к шагу 2 — автопоиск FiveM
      onboardingStep = 2;
      onboardingFivemSearching = true;
      onboardingFivemResult = null;
      try {
        const path = await invoke("auto_find_fivem");
        if (path) {
          fivemPath = path;
          fivemFound = true;
          onboardingFivemResult = 'found';
        } else {
          onboardingFivemResult = 'not_found';
        }
      } catch (e) {
        onboardingFivemResult = 'not_found';
      }
      onboardingFivemSearching = false;
    } else if (onboardingStep === 2) {
      // Переход к шагу 3 — ввод никнейма
      onboardingStep = 3;
    } else if (onboardingStep === 3) {
      // Сохранить никнейм и перейти к шагу 4
      onboardingNicknameSaving = true;
      const name = onboardingNickname.trim() || "Player";
      try {
        await invoke("save_username", { name });
        username = name;
      } catch (e) {}
      try {
        await invoke("complete_onboarding");
      } catch (e) {}
      onboardingNicknameSaving = false;
      onboardingStep = 4;
    } else if (onboardingStep === 4) {
      // Закрыть онбординг
      onboardingActive = false;
    }
  }

  async function onboardingSelectFivem() {
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
        onboardingFivemResult = 'found';
      }
    } catch (e) {}
  }

  async function onboardingDownloadFivem() {
    playClickSound();
    isDownloading = true;
    downloadPercent = 0;
    downloadSize = "";
    downloadError = "";
    try {
      const unlisten = await listen("download-progress", (event) => {
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
      const poll = setInterval(async () => {
        try {
          const installed = await invoke("check_fivem_installed");
          if (installed) {
            clearInterval(poll);
            await loadFivemPath();
            isDownloading = false;
            onboardingFivemResult = 'found';
            statusMessage = "FiveM установлен ✓";
            setTimeout(() => { statusMessage = ""; }, 3000);
          }
        } catch (e) {}
      }, 3000);
    } catch (error) {
      downloadError = `${error}`;
      isDownloading = false;
    }
  }

  // ── Telegram в лаунчере (встроенный webview) ──
  let browserActive = $state(false);

  async function openEmbeddedBrowser(url) {
    try {
      await invoke("create_embedded_webview", { url });
      browserActive = true;
      activeMenu = "news";
    } catch (err) {
      // webview error
      openUrl(url);
    }
  }

  async function closeEmbeddedBrowser() {
    try {
      await invoke("close_embedded_webview");
    } catch (err) {
      // webview close error
    }
    browserActive = false;
  }

  // ── Обработчик меню ─────────────────────────────
  function selectMenu(id) {
    playClickSound();
    if (id === "discord") {
      openUrl("https://discord.gg/zfVss2mNVW");
      return;
    }
    if (id === "news") {
      openEmbeddedBrowser("https://t.me/astrarp5");
      return;
    }
    // Если уходим с новости — закрываем webview
    if (browserActive && id !== "news") {
      closeEmbeddedBrowser();
    }

    // #10 — Анимация перехода
    previousMenu = activeMenu;
    activeMenu = id;
    pageTransition = "fade-in";

    if (id === "settings") {
      loadFivemPath();
    }
  }

  // ── #12 Drag зоны ───────────────────────────────
  // Перетаскивание теперь через data-tauri-drag-region на drag-зоне
  // (нативный механизм Tauri, надёжнее чем startDragging)

  // ── Инициализация ───────────────────────────────
  onMount(() => {
    // Вычисляем DPI масштаб на основе device pixel ratio
    dpiScale = window.devicePixelRatio || 1;

    // Получаем версию приложения из Tauri
    getVersion().then(v => { updateInfo.current_version = v; }).catch(() => {});

    initParticles();
    setTimeout(() => { launcherReady = true; }, 100);

    // #11 — Инициализация звуков через AudioContext
    try {
      const audioCtx = new (window.AudioContext || window.webkitAudioContext)();

      // Звук клика — короткий щелчок (800Hz, 50ms)
      const generateClick = () => {
        const osc = audioCtx.createOscillator();
        const gain = audioCtx.createGain();
        osc.connect(gain);
        gain.connect(audioCtx.destination);
        osc.frequency.value = 800;
        osc.type = 'sine';
        gain.gain.setValueAtTime(0.15, audioCtx.currentTime);
        gain.gain.exponentialRampToValueAtTime(0.001, audioCtx.currentTime + 0.05);
        osc.start(audioCtx.currentTime);
        osc.stop(audioCtx.currentTime + 0.05);
      };
      clickSound = { play: generateClick, currentTime: 0 };

      // Звук наведения — мягкий тон (600Hz, 40ms)
      const generateHover = () => {
        const osc = audioCtx.createOscillator();
        const gain = audioCtx.createGain();
        osc.connect(gain);
        gain.connect(audioCtx.destination);
        osc.frequency.value = 600;
        osc.type = 'sine';
        gain.gain.setValueAtTime(0.25, audioCtx.currentTime);
        gain.gain.exponentialRampToValueAtTime(0.001, audioCtx.currentTime + 0.04);
        osc.start(audioCtx.currentTime);
        osc.stop(audioCtx.currentTime + 0.04);
      };
      hoverSound = { play: generateHover, currentTime: 0 };
    } catch (e) {
      clickSound = null;
      hoverSound = null;
    }

    loadServerStatus();
    loadFivemPath();
    autoFindFivem();
    loadUsername();
    loadServerPing();
    checkUpdates();
    initDiscordRpc();
    checkOnboarding();

    // Автообновление сервера каждые 10 сек (без мерцания — serverLoading не переключается)
    const serverInterval = setInterval(loadServerStatus, 10000);
    const pingInterval = setInterval(loadServerPing, 10000);
    // Проверка обновлений каждые 30 мин
    const updateInterval = setInterval(checkUpdates, 1800000);

    return () => {
      clearInterval(serverInterval);
      clearInterval(pingInterval);
      clearInterval(updateInterval);
    };
  });
</script>

<!-- ── Корневой контейнер ── -->
<svelte:window onmousemove={handleMouseMove} />

<div class="astra-cursor relative w-full h-full min-w-[960px] min-h-[600px] rounded-[5px] overflow-hidden bg-[#0d0d0d] text-white select-none transition-opacity duration-700 {launcherReady ? 'opacity-100' : 'opacity-0'}">

  <!-- ═══════════════════════════════════════════════
       #6 АНИМИРОВАННЫЙ ФОН (частицы + градиент)
       ═══════════════════════════════════════════════ -->
  <Background {parallaxX} {parallaxY} {dpiScale} {particleStyles} />

  <!-- ═══════════════════════════════════════════════
       #7 УВЕДОМЛЕНИЯ (toast)
       ═══════════════════════════════════════════════ -->
  <Notifications {notifications} />

  <!-- ═══════════════════════════════════════════════
       #5 ИНДИКАТОР ОБНОВЛЕНИЯ
       ═══════════════════════════════════════════════ -->
  <UpdateIndicator {updateAvailable} {updateInfo} onOpen={openUpdateModal} {playHoverSound} />

  <!-- ═══════════════════════════════════════════════
       DRAG ZONE — верхняя полоса для перетаскивания окна
       data-tauri-drag-region + fallback startDragging()
       ═══════════════════════════════════════════════ -->
  <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
  <div data-tauri-drag-region role="none" class="absolute top-0 left-0 right-[80px] h-[32px] z-[99] cursor-default"
       onmousedown={() => { if (appWindow) appWindow.startDragging(); }}>
  </div>

  <!-- ═══════════════════════════════════════════════
       КНОПКИ УПРАВЛЕНИЯ ОКНОМ (свернуть / закрыть)
       ═══════════════════════════════════════════════ -->
  <WindowControls onClose={() => { playClickSound(); showCloseConfirm = true; }} {playHoverSound} />

  <!-- ═══════════════════════════════════════════════
       МОДАЛКА ПОДТВЕРЖДЕНИЯ ЗАКРЫТИЯ
       ═══════════════════════════════════════════════ -->
  <CloseConfirmModal show={showCloseConfirm} onClose={() => { showCloseConfirm = false; }} {playClickSound} {playHoverSound} />

  <!-- ═══════════════════════════════════════════════
       МОДАЛКА ПОДТВЕРЖДЕНИЯ ВЫХОДА ИЗ ПРОФИЛЯ
       ═══════════════════════════════════════════════ -->
  <LogoutConfirmModal
    show={showLogoutConfirm}
    onClose={() => { showLogoutConfirm = false; }}
    onLogout={() => { username = "Player"; addNotification("Вы вышли из аккаунта", "info"); }}
    {playClickSound}
    {playHoverSound}
  />

  <!-- ═══════════════════════════════════════════════
       МОДАЛКА ОБНОВЛЕНИЯ
       ═══════════════════════════════════════════════ -->
  <UpdateModal
    show={showUpdateModal}
    {updateInfo}
    {updateDownloading}
    {updateDownloadPercent}
    {updateDownloaded}
    {updateTotal}
    {updateError}
    onStartDownload={startUpdateDownload}
    onClose={closeUpdateModal}
    {playClickSound}
    {playHoverSound}
  />





  <!-- ═══════════════════════════════════════════════
       БОКОВАЯ ПАНЕЛЬ (Sidebar) — из imdex.tsx
       Фон: #1b1b1b, ширина: 157px
       ═══════════════════════════════════════════════ -->
  <Sidebar
    {activeMenu}
    {serverOnline}
    {serverLoading}
    {serverPlayers}
    {serverPing}
    {username}
    {menuItems}
    {selectMenu}
    {playHoverSound}
    {playClickSound}
    onLogout={() => { showLogoutConfirm = true; }}
  />

  <!-- ═══════════════════════════════════════════════
       ОСНОВНАЯ ОБЛАСТЬ (Main Area)
       ═══════════════════════════════════════════════ -->
  <main class="absolute top-0 left-[157px] right-0 bottom-0 flex flex-col items-center justify-center z-10 overflow-hidden">
    {#key activeMenu}
    <div class="flex flex-col items-center justify-center w-full h-full animate-page-enter">

    {#if activeMenu === "settings"}
      <!-- ═══════════════════════════════════════
           СЕКЦИЯ: НАСТРОЙКИ
           ═══════════════════════════════════════ -->
      <Settings
        {fivemPath}
        {fivemFound}
        {updateInfo}
        {updateAvailable}
        {updateChecked}
        {serverOnline}
        {serverPlayers}
        {serverMaxPlayers}
        {selectFivemPath}
        {autoFindFivem}
        {openUpdateModal}
        {playHoverSound}
      />

    {:else if activeMenu === "play"}
      <!-- ═══════════════════════════════════════
           СЕКЦИЯ: ИГРАТЬ (из imdex.tsx дизайн)
           ═══════════════════════════════════════ -->
      <PlaySection
        {launchPhase}
        {launchProgress}
        {launchPhases}
        {fivemFound}
        {isLaunching}
        {serverOnline}
        {isDownloading}
        {downloadPercent}
        {downloadSize}
        {downloadError}
        {precacheDownloading}
        {precacheExtracting}
        {precachePercent}
        {playLabel}
        {statusMessage}
        {handlePlay}
        {downloadAndInstall}
        {checkFivemAgain}
        {playHoverSound}
      />

    {:else}
      <!-- ═══════════════════════════════════════
           ЗАГЛУШКА (Магазин/Форум)
           ═══════════════════════════════════════ -->
      <div class="z-10 text-center">
        <div class="mb-4 flex justify-center">
          {#if activeMenu === 'store'}
            <svg class="w-16 h-16 text-white/30" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1" stroke-linecap="round" stroke-linejoin="round"><path d="M0.75 14.75H14.75M10.08 4.64C10.08 5.26 10.33 5.85 10.77 6.29C11.2 6.73 11.8 6.97 12.42 6.97C13.04 6.97 13.63 6.73 14.07 6.29C14.5 5.85 14.75 5.26 14.75 4.64V3.86L13.19 0.75H2.31L0.75 3.86V4.64C0.75 5.26 1 5.85 1.43 6.29C1.87 6.73 2.46 6.97 3.08 6.97C3.7 6.97 4.3 6.73 4.73 6.29C5.17 5.85 5.42 5.26 5.42 4.64M5.42 3.86V4.64C5.42 5.26 5.66 5.85 6.1 6.29C6.54 6.73 7.13 6.97 7.75 6.97C8.37 6.97 8.96 6.73 9.4 6.29C9.84 5.85 10.08 5.26 10.08 4.64V3.86M2.31 14.75V6.86M13.19 14.75V6.86M5.42 14.75V11.64C5.42 11.23 5.58 10.83 5.87 10.54C6.16 10.25 6.56 10.08 6.97 10.08H8.53C8.94 10.08 9.34 10.25 9.63 10.54C9.92 10.83 10.08 11.23 10.08 11.64V14.75"/></svg>
          {:else if activeMenu === 'forum'}
            <svg class="w-16 h-16 text-white/30" viewBox="0 0 16 15" fill="none" stroke="currentColor" stroke-width="1" stroke-linecap="round" stroke-linejoin="round"><path d="M4.74 4.67H10.96M4.74 7.73H9.41M5.52 11.56H3.18C2.56 11.56 1.97 11.31 1.53 10.88C1.1 10.45 0.85 9.87 0.85 9.26V3.14C0.85 2.54 1.1 1.95 1.53 1.52C1.97 1.09 2.56 0.85 3.18 0.85H12.52C13.14 0.85 13.73 1.09 14.17 1.52C14.6 1.95 14.85 2.54 14.85 3.14V9.26C14.85 9.87 14.6 10.45 14.17 10.88C13.73 11.31 13.14 11.56 12.52 11.56H10.18L7.85 13.85L5.52 11.56Z"/></svg>
          {:else}
            <svg class="w-16 h-16 text-white/30" viewBox="0 0 14 13" fill="currentColor"><path d="M12.66 4.59C11.4 4.59 10.88 3.7 11.51 2.62C11.87 2 11.66 1.2 11.02 0.84L9.81 0.16C9.26 -0.17 8.54 0.03 8.21 0.57L8.14 0.7C7.51 1.78 6.47 1.78 5.84 0.7L5.76 0.57C5.44 0.03 4.73 -0.17 4.18 0.16L2.97 0.84C2.33 1.2 2.11 2 2.48 2.63C3.11 3.7 2.6 4.59 1.33 4.59C0.6 4.59 0 5.17 0 5.89V7.11C0 7.82 0.59 8.41 1.33 8.41C2.6 8.41 3.11 9.3 2.48 10.38C2.11 11 2.33 11.8 2.97 12.16L4.18 12.84C4.73 13.17 5.44 12.97 5.77 12.43L5.85 12.3C6.48 11.22 7.51 11.22 8.15 12.3L8.23 12.43C8.56 12.97 9.27 13.17 9.82 12.84L11.03 12.16C11.67 11.8 11.89 11 11.52 10.38C10.89 9.3 11.4 8.41 12.67 8.41C13.4 8.41 14 7.83 14 7.11V5.89C14 5.18 13.4 4.59 12.66 4.59ZM7 8.74C5.74 8.74 4.72 7.73 4.72 6.5C4.72 5.27 5.74 4.26 7 4.26C8.25 4.26 9.27 5.27 9.27 6.5C9.27 7.73 8.25 8.74 7 8.74Z"/></svg>
          {/if}
        </div>
        <h2 class="text-xl font-bold mb-2 text-reveal" style="font-family: 'Proxima Nova Bold', sans-serif;">
          {menuItems.find(m => m.id === activeMenu)?.label || "Раздел"}
        </h2>
        <p class="text-sm text-gray-500">Раздел в разработке</p>
      </div>
    {/if}
    </div>
    {/key}
  </main>

  <!-- ═══════════════════════════════════════════════
       ОНБОРДИНГ (первый запуск)
       ═══════════════════════════════════════════════ -->
  <OnboardingModal
    {onboardingActive}
    {onboardingStep}
    {onboardingFivemSearching}
    {onboardingFivemResult}
    {onboardingNickname}
    {onboardingNicknameSaving}
    {fivemPath}
    {isDownloading}
    {downloadPercent}
    {downloadSize}
    {downloadError}
    {onboardingNext}
    {onboardingSelectFivem}
    {onboardingDownloadFivem}
    {playHoverSound}
    onNicknameChange={(val) => { onboardingNickname = val; }}
  />
</div>
