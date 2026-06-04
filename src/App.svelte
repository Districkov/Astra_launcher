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
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { open } from "@tauri-apps/plugin-dialog";
  import { open as openUrl } from "@tauri-apps/plugin-shell";
  import { listen } from "@tauri-apps/api/event";
  import { onMount } from "svelte";

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
  let updateInfo = $state({ current_version: "1.6.6", latest_version: "", release_notes: "" });
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

  // #10 — Анимация переходов
  let pageTransition = $state("fade-in");

  // #11 — Звуки кликов и наведения
  let clickSound = null;
  let hoverSound = null;

  // #2 — Пинг сервера
  let serverPing = $state(0);

  // #14 — Музыка в лаунчере
  // #13 — Discord Rich Presence (флаг)
  let discordRpcActive = $state(false);

  let showCloseConfirm = $state(false);

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
      // Обновление скачано и установлено — перезапускаем
      addNotification("✅ Обновление установлено! Перезапуск...", "success");
      setTimeout(() => {
        pendingUpdate.installAndRestart();
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
      await pendingUpdate.installAndRestart();
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
  <div class="absolute inset-0 z-0 overflow-hidden pointer-events-none transition-transform duration-300 ease-out"
       style="transform: translate({parallaxX * 0.3}px, {parallaxY * 0.3}px);">
    <!-- Плавающие градиентные блобы -->
    <div class="animated-blob blob-1"></div>
    <div class="animated-blob blob-2"></div>
    <div class="animated-blob blob-3"></div>
    <!-- Частицы (звёзды) -->
    {#each particleStyles as ps, i}
      <div class="particle" style="
        left: {ps.left};
        top: {ps.top};
        animation-delay: {ps.delay};
        animation-duration: {ps.duration};
        width: {ps.width};
        height: {ps.height};
      "></div>
    {/each}
  </div>

  <!-- ═══════════════════════════════════════════════
       #7 УВЕДОМЛЕНИЯ (toast)
       ═══════════════════════════════════════════════ -->
  <div class="absolute top-[40px] right-[16px] z-[200] flex flex-col gap-2 pointer-events-none">
    {#each notifications as notif (notif.id)}
      <div class="pointer-events-auto px-4 py-2.5 rounded-lg text-sm shadow-lg backdrop-blur-sm transition-all duration-300 animate-slide-in
        {notif.type === 'success' ? 'bg-green-500/20 border border-green-500/30 text-green-300' :
         notif.type === 'error' ? 'bg-red-500/20 border border-red-500/30 text-red-300' :
         notif.type === 'update' ? 'bg-[#f64a46]/20 border border-[#f64a46]/30 text-[#ff8c8c]' :
         'bg-white/10 border border-white/10 text-white/70'}">
        {notif.message}
      </div>
    {/each}
  </div>

  <!-- ═══════════════════════════════════════════════
       #5 ИНДИКАТОР ОБНОВЛЕНИЯ
       ═══════════════════════════════════════════════ -->
  {#if updateAvailable}
    <button
      class="absolute top-[40px] left-[170px] z-[80] flex items-center gap-2 px-3 py-1.5 rounded-lg bg-[#f64a46]/15 border border-[#f64a46]/20 cursor-pointer hover:bg-[#f64a46]/25 transition-colors"
      onclick={openUpdateModal}
      onmouseenter={playHoverSound}
      aria-label="Доступно обновление"
    >
      <div class="w-2 h-2 rounded-full bg-[#f64a46] update-badge-dot"></div>
      <span class="text-xs text-[#ff8c8c]">Обновление v{updateInfo.latest_version}</span>
    </button>
  {/if}

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
  <div class="absolute top-0 right-0 z-[100] flex items-center gap-0">
    <button
      class="w-10 h-8 flex items-center justify-center text-white/40 hover:text-white hover:bg-white/10 transition-colors"
      onclick={(e) => { e.stopPropagation(); invoke("window_minimize"); }}
      onmouseenter={playHoverSound}
      title="Свернуть"
    >
      <svg width="12" height="1" viewBox="0 0 12 1" fill="currentColor"><rect width="12" height="1"/></svg>
    </button>
    <button
      class="w-10 h-8 flex items-center justify-center text-white/40 hover:text-[#f64a46] hover:bg-white/10 transition-colors"
      onclick={(e) => { e.stopPropagation(); playClickSound(); showCloseConfirm = true; }}
      onmouseenter={playHoverSound}
      title="Закрыть"
    >
      <svg width="10" height="10" viewBox="0 0 10 10" fill="none" stroke="currentColor" stroke-width="1.2"><line x1="1" y1="1" x2="9" y2="9"/><line x1="9" y1="1" x2="1" y2="9"/></svg>
    </button>
  </div>

  <!-- ═══════════════════════════════════════════════
       МОДАЛКА ПОДТВЕРЖДЕНИЯ ЗАКРЫТИЯ
       ═══════════════════════════════════════════════ -->
  {#if showCloseConfirm}
    <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
    <div class="absolute inset-0 z-[300] flex items-center justify-center bg-black/70 backdrop-blur-sm animate-fade-in" onclick={() => { showCloseConfirm = false; }}>
      <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
      <div class="bg-[#1b1b1b] border border-white/10 rounded-xl px-8 py-6 max-w-xs w-full shadow-2xl" onclick={(e) => e.stopPropagation()}>
        <!-- Логотип ASTRA + красная полоска -->
        <div class="text-center mb-5">
          <div class="text-white tracking-[-0.8px] leading-none" style="font-family: 'Armor Piercing 2.0 BB', 'Impact', sans-serif; font-size: 28px;">
            ASTRA
          </div>
          <div class="mt-1.5 mx-auto w-[50px] h-[3px] bg-[#f64a46] rounded-full"></div>
        </div>
        <h3 class="text-lg text-white mb-1 text-center" style="font-family: 'Proxima Nova Bold', sans-serif; font-weight: 700; letter-spacing: -0.36px;">
          Закрыть лаунчер?
        </h3>
        <p class="text-sm text-white/30 mb-5 text-center" style="font-family: 'Proxima Nova Semibold', sans-serif;">
          Серверный кеш и прогресс сохранены
        </p>
        <div class="flex gap-3">
          <button
            class="flex-1 px-4 py-2.5 rounded-lg bg-[#f64a46] hover:bg-[#ff5a56] active:scale-[0.97] text-sm text-white font-medium transition-all btn-ripple btn-bounce glow-hover"
            style="font-family: 'Proxima Nova Semibold', sans-serif; font-weight: 600;"
            onclick={(e) => { e.stopPropagation(); playClickSound(); invoke("window_close"); }}
            onmouseenter={playHoverSound}
          >
            Да, закрыть
          </button>
          <button
            class="flex-1 px-4 py-2.5 rounded-lg bg-white/5 hover:bg-white/10 text-sm text-white/50 hover:text-white/70 font-medium transition-all border border-white/5 btn-ripple"
            style="font-family: 'Proxima Nova Semibold', sans-serif; font-weight: 600;"
            onclick={(e) => { e.stopPropagation(); playClickSound(); showCloseConfirm = false; }}
            onmouseenter={playHoverSound}
          >
            Отмена
          </button>
        </div>
      </div>
    </div>
  {/if}

  <!-- ═══════════════════════════════════════════════
       МОДАЛКА ОБНОВЛЕНИЯ
       ═══════════════════════════════════════════════ -->
  {#if showUpdateModal}
    <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
    <div class="absolute inset-0 z-[300] flex items-center justify-center bg-black/70 backdrop-blur-sm animate-fade-in" onclick={closeUpdateModal}>
      <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
      <div role="dialog" class="bg-[#1b1b1b] border border-white/10 rounded-xl px-8 py-6 max-w-md w-full shadow-2xl" onclick={(e) => e.stopPropagation()}>
        <!-- Логотип ASTRA -->
        <div class="text-center mb-4">
          <div class="text-white tracking-[-0.8px] leading-none" style="font-family: 'Armor Piercing 2.0 BB', 'Impact', sans-serif; font-size: 28px;">
            ASTRA
          </div>
          <div class="mt-1.5 mx-auto w-[50px] h-[3px] bg-[#f64a46] rounded-full"></div>
        </div>

        <h3 class="text-lg text-white mb-1 text-center" style="font-family: 'Proxima Nova Bold', sans-serif; font-weight: 700; letter-spacing: -0.36px;">
          🔄 Доступно обновление
        </h3>
        <p class="text-sm text-white/40 mb-4 text-center" style="font-family: 'Proxima Nova Semibold', sans-serif;">
          v{updateInfo.current_version} → v{updateInfo.latest_version}
        </p>

        {#if updateInfo.release_notes}
          <div class="mb-4 p-3 rounded-lg bg-white/5 border border-white/5 max-h-[120px] overflow-y-auto">
            <p class="text-xs text-white/50 whitespace-pre-wrap">{updateInfo.release_notes}</p>
          </div>
        {/if}

        {#if updateError}
          <div class="mb-4 p-3 rounded-lg bg-red-500/10 border border-red-500/20">
            <p class="text-xs text-red-300">❌ {updateError}</p>
          </div>
        {/if}

        {#if updateDownloading}
          <!-- Прогресс скачивания -->
          <div class="mb-4">
            <div class="flex justify-between text-xs text-white/40 mb-1">
              <span>Скачивание обновления...</span>
              <span>{updateDownloadPercent}%</span>
            </div>
            <div class="w-full h-2 bg-white/10 rounded-full overflow-hidden">
              <div class="h-full bg-[#f64a46] rounded-full transition-all duration-300 progress-glow" style="width: {updateDownloadPercent}%"></div>
            </div>
            {#if updateDownloaded > 0}
              <p class="text-[10px] text-white/20 mt-1">
                {(updateDownloaded / 1048576).toFixed(1)} МБ{updateTotal > 0 ? ` / ${(updateTotal / 1048576).toFixed(1)} МБ` : ''}
              </p>
            {/if}
          </div>
          <div class="flex gap-3">
            <button
              class="flex-1 px-4 py-2.5 rounded-lg bg-white/5 hover:bg-white/10 text-sm text-white/50 hover:text-white/70 font-medium transition-all border border-white/5 btn-ripple"
              style="font-family: 'Proxima Nova Semibold', sans-serif; font-weight: 600;"
              onclick={closeUpdateModal}
              onmouseenter={playHoverSound}
            >
              Свернуть
            </button>
          </div>
        {:else}
          <!-- Кнопка скачивания -->
          <div class="flex gap-3">
            <button
              class="flex-1 px-4 py-2.5 rounded-lg bg-[#f64a46] hover:bg-[#ff5a56] active:scale-[0.97] text-sm text-white font-medium transition-all btn-ripple btn-bounce glow-hover"
              style="font-family: 'Proxima Nova Semibold', sans-serif; font-weight: 600;"
              onclick={startUpdateDownload}
              onmouseenter={playHoverSound}
            >
              Обновить
            </button>
            <button
              class="flex-1 px-4 py-2.5 rounded-lg bg-white/5 hover:bg-white/10 text-sm text-white/50 hover:text-white/70 font-medium transition-all border border-white/5 btn-ripple"
              style="font-family: 'Proxima Nova Semibold', sans-serif; font-weight: 600;"
              onclick={closeUpdateModal}
              onmouseenter={playHoverSound}
            >
              Позже
            </button>
          </div>
        {/if}
      </div>
    </div>
  {/if}

  <!-- ═══════════════════════════════════════════════
       ФОН: Декоративные элементы (из assets)
       ═══════════════════════════════════════════════ -->

  <!-- Subtract1 — красная подсветка справа сверху -->
  <svg class="absolute top-0 left-[504px] w-[456px] h-[400px] pointer-events-none select-none z-[6] transition-transform duration-300 ease-out"
       style="transform: translate({parallaxX * 0.6}px, {parallaxY * 0.6}px);" viewBox="0 0 456 400" fill="none">
    <defs><filter id="glow1" x="-200" y="-200" width="900" height="900" filterUnits="userSpaceOnUse">
      <feGaussianBlur stdDeviation="100"/><feComposite in2="hardAlpha" operator="out"/>
      <feColorMatrix type="matrix" values="0 0 0 0 0.965 0 0 0 0 0.29 0 0 0 0 0.275 0 0 0 1 0"/>
      <feBlend mode="normal" in2="BackgroundImageFix" result="effect1"/><feBlend mode="normal" in="SourceGraphic" in2="effect1" result="shape"/>
    </filter></defs>
    <g filter="url(#glow1)"><path d="M367.5 200C460 200 535 125 535 32.5C535 -60 460 -135 367.5 -135C275 -135 200 -60 200 32.5C200 125 275 200 367.5 200ZM368 164.5C295.6 164.5 236.8 105.7 236.8 33.2C236.8 -39.4 295.6 -98.2 368 -98.2C440.7 -98.2 499.5 -39.4 499.5 33.2C499.5 105.7 440.7 164.5 368 164.5Z" fill="#F64A46"/></g>
  </svg>

  <!-- Subtract — красная подсветка слева снизу -->
  <svg class="absolute top-[282px] left-0 w-[642px] h-[358px] pointer-events-none select-none z-[6] transition-transform duration-300 ease-out"
       style="transform: translate({parallaxX * 0.5}px, {parallaxY * 0.5}px);" viewBox="0 0 647 386" fill="none">
    <defs><filter id="glow2" x="-200" y="-200" width="1100" height="900" filterUnits="userSpaceOnUse">
      <feGaussianBlur stdDeviation="100"/><feComposite in2="hardAlpha" operator="out"/>
      <feColorMatrix type="matrix" values="0 0 0 0 0.965 0 0 0 0 0.29 0 0 0 0 0.275 0 0 0 1 0"/>
      <feBlend mode="normal" in2="BackgroundImageFix" result="effect1"/><feBlend mode="normal" in="SourceGraphic" in2="effect1" result="shape"/>
    </filter></defs>
    <g filter="url(#glow2)"><path d="M279.5 535C372 535 447 460 447 367.5C447 275 372 200 279.5 200C187 200 112 275 112 367.5C112 460 187 535 279.5 535ZM280 499.5C207.6 499.5 148.8 440.7 148.8 368.2C148.8 295.6 207.6 236.8 280 236.8C352.7 236.8 411.5 295.6 411.5 368.2C411.5 440.7 352.7 499.5 280 499.5Z" fill="#F64A46"/></g>
  </svg>

  <!-- GTAV персонаж — выше текста ASTRA и звёзд, но ниже кнопки играть -->
  <img src="/GTAV-33-4.png.png" alt="" class="absolute top-4 left-[16.5%] w-[83.5%] h-[104%] object-cover pointer-events-none select-none z-[5] transition-transform duration-300 ease-out"
       style="transform: translate({parallaxX * 0.8}px, {parallaxY * 0.8}px);" aria-hidden="true" />

  <!-- Большой текст ASTRA на фоне -->
  <div class="pointer-events-none select-none"
       style="position: absolute; top: {108 / dpiScale}px; left: {230 / dpiScale}px;">
    <div class="pointer-events-none select-none"
         style="font-family: 'Armor Piercing 2.0 BB', 'Impact', sans-serif; font-size: {280 / dpiScale}px; font-weight: normal; color: transparent; -webkit-text-stroke: 1px rgba(255,255,255,0.05); letter-spacing: {-8 / dpiScale}px; line-height: normal; white-space: nowrap;">
      ASTRA
    </div>
  </div>

  <!-- Star 23 — звезда справа снизу -->
  <svg class="absolute top-[410px] left-[702px] w-[258px] h-[230px] pointer-events-none select-none transition-transform duration-300 ease-out"
       style="transform: translate({parallaxX * 1.2}px, {parallaxY * 1.2}px);" viewBox="0 0 259 230" fill="none">
    <path d="M95.1 301.5C94.06 325 123.66 336.1 138.3 317.7L182.8 261.8C189.2 253.8 200.1 250.8 209.7 254.4L276.6 279.5C298.6 287.7 318.3 263 305.4 243.4L266 183.8C260.3 175.2 260.8 164 267.2 155.9L311.8 100C326.4 81.7 309 55.3 286.3 61.5L217.5 80.6C207.5 83.3 197 79.4 191.3 70.8L151.9 11.2C139 -8.4 108.5 0 107.5 23.5L104.3 94.8C103.8 105.1 96.8 113.9 86.9 116.7L18 135.7C-4.6 142 -6 173.5 16 181.8L82.9 206.9C92.5 210.5 98.7 219.9 98.3 230.2L95.1 301.5Z" stroke="white" stroke-opacity="0.3" stroke-width="0.5"/>
  </svg>

  <!-- Star 24 — звезда сверху -->
  <svg class="absolute top-0 left-[106px] w-[323px] h-[156px] pointer-events-none select-none transition-transform duration-300 ease-out"
       style="transform: translate({parallaxX * 1.4}px, {parallaxY * 1.4}px);" viewBox="0 0 324 157" fill="none">
    <path d="M201.2 148.6C217.6 165.4 246 151.5 242.6 128.2L232.5 57.5C231 47.3 236.2 37.3 245.5 32.8L309.6 1.3C330.6 -9.1 326.1 -40.4 303 -44.4L232.6 -56.6C222.5 -58.3 214.6 -66.4 213.1 -76.6L202.9 -147.3C199.6 -170.5 168.5 -175.9 157.5 -155.1L124.2 -91.9C119.4 -82.8 109.3 -77.9 99.1 -79.6L28.7 -91.8C5.6 -95.8 -9.2 -67.8 7.2 -51L57 0.2C64.2 7.6 65.8 18.8 61 27.9L27.7 91.1C16.7 111.9 38.7 134.5 59.8 124.1L123.9 92.6C133.2 88.1 144.3 90 151.4 97.4L201.2 148.6Z" stroke="white" stroke-opacity="0.3" stroke-width="0.5"/>
  </svg>



  <!-- ═══════════════════════════════════════════════
       БОКОВАЯ ПАНЕЛЬ (Sidebar) — из imdex.tsx
       Фон: #1b1b1b, ширина: 157px
       ═══════════════════════════════════════════════ -->
  <aside class="absolute top-0 left-0 w-[157px] h-full bg-[#1b1b1b] rounded-l-[5px] z-20 flex flex-col">

    <!-- Логотип ASTRA (drag-зона) -->
    <div data-tauri-drag-region role="none" class="pt-[26px] pl-[44px] cursor-default"
         onmousedown={() => { if (appWindow) appWindow.startDragging(); }}>
       <div class="text-white tracking-[-0.8px] leading-none"
         style="font-family: 'Armor Piercing 2.0 BB', 'Impact', sans-serif; font-size: clamp(32px, 4.2vw, 40px);">
        ASTRA
      </div>
      <!-- Красная полоска под логотипом -->
      <div class="-mt-[3px] w-[65px] h-[3px] bg-[#f64a46] rounded-[21px]"></div>
    </div>

    <!-- Статус онлайна -->
    <div class="mt-[40px] pl-[31px] flex items-center gap-2">
      {#if serverLoading}
        <div class="w-3 h-3 rounded-md bg-yellow-500/20 flex items-center justify-center">
          <div class="w-2 h-2 rounded bg-yellow-500 animate-pulse"></div>
        </div>
        <div class="flex flex-col gap-1.5">
          <div class="w-14 h-2.5 rounded bg-white/10 animate-skeleton"></div>
          <div class="w-8 h-2 rounded bg-white/5 animate-skeleton"></div>
        </div>
      {:else if serverOnline}
        <div class="w-3 h-3 rounded-md bg-green-500/20 flex items-center justify-center">
          <div class="w-2 h-2 rounded bg-[#15ff00]"></div>
        </div>
        <span class="text-sm" style="font-family: 'Proxima Nova Bold', sans-serif; font-weight: 700; letter-spacing: -0.04px;">
          <span class="text-white/30">Онлайн:</span>
          <span class="text-white"> {serverPlayers}</span>
          {#if serverPing > 0}
            <span class="text-white/20 ml-1">|</span>
            <span class="text-white/30 ml-1" style="font-size: 11px;">
              {#if serverPing < 50}
                <span class="text-green-400">{serverPing}ms</span>
              {:else if serverPing < 100}
                <span class="text-yellow-400">{serverPing}ms</span>
              {:else}
                <span class="text-red-400">{serverPing}ms</span>
              {/if}
            </span>
          {/if}
        </span>
      {:else}
        <div class="w-3 h-3 rounded-md bg-red-500/20 flex items-center justify-center">
          <div class="w-2 h-2 rounded bg-red-500"></div>
        </div>
        <span class="text-sm text-red-400" style="font-family: 'Proxima Nova Bold', sans-serif; font-weight: 700;">
          Офлайн
        </span>
      {/if}

    </div>

    <!-- Навигация -->
    <nav class="mt-[35px] flex-1">
      {#each menuItems as item, i}
        {#if i > 0 && item.group !== menuItems[i - 1].group}
          <div class="my-3"></div>
        {/if}
        <button
          class="menu-item w-full flex items-center gap-3 pl-[34px] h-[42px] text-[15px] transition-colors duration-150 relative
            {activeMenu === item.id ? 'text-white menu-item-active' : 'text-white/30 hover:text-white/60 menu-item-hover'}"
          style="font-family: 'Proxima Nova Semibold', sans-serif; font-weight: 600; letter-spacing: -0.30px;
            {activeMenu === item.id ? 'background: linear-gradient(90deg, rgba(246,74,70,0.5) 0%, rgba(0,0,0,0.1) 100%)' : ''}"
          onclick={() => selectMenu(item.id)}
          onmouseenter={playHoverSound}
        >
          {#if item.icon === 'play'}
            <svg class="w-3 h-3" viewBox="0 0 16 17" fill="currentColor"><polygon points="0,0 16,8.5 0,17" /></svg>
          {:else if item.icon === 'store'}
            <svg class="w-4 h-4" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M0.75 14.75H14.75M10.08 4.64C10.08 5.26 10.33 5.85 10.77 6.29C11.2 6.73 11.8 6.97 12.42 6.97C13.04 6.97 13.63 6.73 14.07 6.29C14.5 5.85 14.75 5.26 14.75 4.64V3.86L13.19 0.75H2.31L0.75 3.86V4.64C0.75 5.26 1 5.85 1.43 6.29C1.87 6.73 2.46 6.97 3.08 6.97C3.7 6.97 4.3 6.73 4.73 6.29C5.17 5.85 5.42 5.26 5.42 4.64M5.42 3.86V4.64C5.42 5.26 5.66 5.85 6.1 6.29C6.54 6.73 7.13 6.97 7.75 6.97C8.37 6.97 8.96 6.73 9.4 6.29C9.84 5.85 10.08 5.26 10.08 4.64V3.86M2.31 14.75V6.86M13.19 14.75V6.86M5.42 14.75V11.64C5.42 11.23 5.58 10.83 5.87 10.54C6.16 10.25 6.56 10.08 6.97 10.08H8.53C8.94 10.08 9.34 10.25 9.63 10.54C9.92 10.83 10.08 11.23 10.08 11.64V14.75"/></svg>
          {:else if item.icon === 'news'}
            <svg class="w-4 h-4" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M11.25 2.5H13.88C14.11 2.5 14.33 2.59 14.49 2.76C14.66 2.92 14.75 3.14 14.75 3.38V13C14.75 13.46 14.57 13.91 14.24 14.24C13.91 14.57 13.46 14.75 13 14.75M11.25 2.5V13C11.25 13.46 11.43 13.91 11.76 14.24C12.09 14.57 12.54 14.75 13 14.75M11.25 2.5V1.63C11.25 1.39 11.16 1.17 10.99 1.01C10.83 0.84 10.61 0.75 10.38 0.75H1.63C1.39 0.75 1.17 0.84 1.01 1.01C0.84 1.17 0.75 1.39 0.75 1.63V12.13C0.75 12.82 1.03 13.49 1.52 13.98C2.01 14.47 2.68 14.75 3.38 14.75H13M4.25 4.25H7.75M4.25 7.75H7.75M4.25 11.25H7.75"/></svg>
          {:else if item.icon === 'forum'}
            <svg class="w-4 h-4" viewBox="0 0 16 15" fill="none" stroke="currentColor" stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round"><path d="M4.74 4.67H10.96M4.74 7.73H9.41M5.52 11.56H3.18C2.56 11.56 1.97 11.31 1.53 10.88C1.1 10.45 0.85 9.87 0.85 9.26V3.14C0.85 2.54 1.1 1.95 1.53 1.52C1.97 1.09 2.56 0.85 3.18 0.85H12.52C13.14 0.85 13.73 1.09 14.17 1.52C14.6 1.95 14.85 2.54 14.85 3.14V9.26C14.85 9.87 14.6 10.45 14.17 10.88C14.73 11.31 13.14 11.56 12.52 11.56H10.18L7.85 13.85L5.52 11.56Z"/></svg>
          {:else if item.icon === 'discord'}
            <svg class="w-3.5 h-3" viewBox="0 0 14 11" fill="currentColor"><path d="M11.86 0.92C10.95 0.49 9.98 0.18 8.97 0C8.85 0.23 8.7 0.54 8.6 0.78C7.52 0.62 6.46 0.62 5.4 0.78C5.3 0.54 5.15 0.23 5.03 0C4.01 0.18 3.04 0.49 2.14 0.92C0.31 3.73 -0.19 6.46 0.06 9.16C1.27 10.08 2.45 10.63 3.6 11C3.89 10.6 4.14 10.18 4.36 9.73C3.94 9.57 3.54 9.37 3.17 9.14C3.27 9.07 3.36 8.99 3.46 8.91C5.76 10 8.27 10 10.54 8.91C10.64 8.99 10.74 9.07 10.83 9.14C10.46 9.37 10.05 9.57 9.64 9.73C9.86 10.18 10.11 10.6 10.4 11C11.55 10.63 12.73 10.08 13.94 9.16C14.23 6.03 13.44 3.32 11.86 0.92ZM4.67 7.5C3.98 7.5 3.42 6.84 3.42 6.05C3.42 5.25 3.97 4.59 4.67 4.59C5.38 4.59 5.95 5.25 5.93 6.05C5.93 6.84 5.38 7.5 4.67 7.5ZM9.33 7.5C8.63 7.5 8.07 6.84 8.07 6.05C8.07 5.25 8.62 4.59 9.33 4.59C10.03 4.59 10.6 5.25 10.58 6.05C10.58 6.84 10.03 7.5 9.33 7.5Z"/></svg>
          {:else if item.icon === 'settings'}
            <svg class="w-3.5 h-3" viewBox="0 0 14 13" fill="currentColor"><path d="M12.66 4.59C11.4 4.59 10.88 3.7 11.51 2.62C11.87 2 11.66 1.2 11.02 0.84L9.81 0.16C9.26 -0.17 8.54 0.03 8.21 0.57L8.14 0.7C7.51 1.78 6.47 1.78 5.84 0.7L5.76 0.57C5.44 0.03 4.73 -0.17 4.18 0.16L2.97 0.84C2.33 1.2 2.11 2 2.48 2.63C3.11 3.7 2.6 4.59 1.33 4.59C0.6 4.59 0 5.17 0 5.89V7.11C0 7.82 0.59 8.41 1.33 8.41C2.6 8.41 3.11 9.3 2.48 10.38C2.11 11 2.33 11.8 2.97 12.16L4.18 12.84C4.73 13.17 5.44 12.97 5.77 12.43L5.85 12.3C6.48 11.22 7.51 11.22 8.15 12.3L8.23 12.43C8.56 12.97 9.27 13.17 9.82 12.84L11.03 12.16C11.67 11.8 11.89 11 11.52 10.38C10.89 9.3 11.4 8.41 12.67 8.41C13.4 8.41 14 7.83 14 7.11V5.89C14 5.18 13.4 4.59 12.66 4.59ZM7 8.74C5.74 8.74 4.72 7.73 4.72 6.5C4.72 5.27 5.74 4.26 7 4.26C8.25 4.26 9.27 5.27 9.27 6.5C9.27 7.73 8.25 8.74 7 8.74Z"/></svg>
          {/if}
          <span class="whitespace-nowrap">{item.label}</span>
        </button>
      {/each}
    </nav>

    <!-- Разделитель -->
    <div class="mx-4 mb-2 h-px bg-white/10"></div>

    <!-- #9 — Профиль пользователя (имя из сервера/FiveM) -->
    <div class="px-4 pb-3 flex flex-col items-center">
      <div class="flex items-center gap-2 mb-2">
        <svg class="w-6 h-6 text-white/30" viewBox="0 0 22 25" fill="currentColor"><path d="M20.09 4.81L12.89 0.53C11.72 -0.18 10.27 -0.18 9.08 0.53L1.9 4.81C0.73 5.51 0 6.81 0 8.23V16.78C0 18.18 0.73 19.48 1.9 20.19L9.09 24.48C10.27 25.18 11.72 25.18 12.91 24.48L20.1 20.19C21.27 19.49 22 18.19 22 16.78V8.23C21.99 6.81 21.26 5.53 20.09 4.81ZM10.99 6.68C12.56 6.68 13.82 7.98 13.82 9.59C13.82 11.2 12.56 12.5 10.99 12.5C9.43 12.5 8.17 11.2 8.17 9.59C8.17 7.99 9.43 6.68 10.99 6.68ZM14.24 18.33H7.75C6.77 18.33 6.2 17.2 6.74 16.36C7.57 15.1 9.17 14.25 10.99 14.25C12.82 14.25 14.42 15.1 15.24 16.36C15.79 17.19 15.21 18.33 14.24 18.33Z"/></svg>
        <p class="text-sm text-white" style="font-family: 'Proxima Nova Bold', sans-serif; font-weight: 700; letter-spacing: -0.28px;">
          {username}
        </p>
      </div>
      <button
        class="px-4 py-1 text-xs text-white/30 hover:text-white/60 hover:bg-white/5 rounded transition-colors btn-ripple"
        onclick={(e) => { e.stopPropagation(); playClickSound(); username = "Player"; addNotification("Вы вышли из аккаунта", "info"); }}
        onmouseenter={playHoverSound}
      >
        Выйти
      </button>
    </div>
  </aside>

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
      <div class="w-full max-w-lg px-8 z-10">
        <h2 class="text-2xl mb-6 text-reveal" style="font-family: 'Proxima Nova Bold', sans-serif; font-weight: 700; letter-spacing: -0.48px;">Настройки</h2>

        <!-- Путь к FiveM -->
        <div class="settings-card bg-white/5 rounded-lg p-5 mb-4 stagger-1">
          <h3 class="text-sm font-semibold text-white/70 mb-3" style="font-family: 'Proxima Nova Semibold', sans-serif;">Путь к FiveM</h3>
          <p class="text-xs text-white/30 mb-3">
            Укажите расположение FiveM.exe, если лаунчер не нашёл его автоматически.
          </p>
          <div class="flex items-center gap-2 mb-2">
            <div class="flex-1 bg-black/40 rounded-lg px-3 py-2 text-xs text-white/50 truncate border border-white/5">
              {fivemPath || "Не найден"}
            </div>
            <button
              class="px-4 py-2 bg-[#f64a46] hover:bg-[#ff5a56] rounded-lg text-xs font-medium transition-colors btn-ripple btn-bounce glow-hover"
              style="font-family: 'Proxima Nova Semibold', sans-serif;"
              onclick={selectFivemPath}
              onmouseenter={playHoverSound}
            >
              Выбрать
            </button>
          </div>
          <div class="flex items-center gap-2 mt-2">
            <button
              class="px-4 py-2 bg-white/5 hover:bg-white/10 rounded-lg text-xs font-medium transition-colors border border-white/5 text-white/50 hover:text-white/70 btn-ripple"
              style="font-family: 'Proxima Nova Semibold', sans-serif;"
              onclick={autoFindFivem}
              onmouseenter={playHoverSound}
            >
              Найти автоматически
            </button>
          </div>
          {#if fivemFound}
            <p class="text-[10px] text-[#15ff00]/70">✓ FiveM.exe найден</p>
          {:else}
            <p class="text-[10px] text-[#f64a46]/70">✗ FiveM.exe не найден — выберите вручную</p>
          {/if}
        </div>

        <!-- Информация -->
        <div class="settings-card bg-white/5 rounded-lg p-5 stagger-2">
          <h3 class="text-sm font-semibold text-white/70 mb-3" style="font-family: 'Proxima Nova Semibold', sans-serif;">О лаунчере</h3>
          <p class="text-xs text-white/60">ASTRA Launcher v{updateInfo.current_version || '1.3.0'}</p>
          <p class="text-xs text-white/60 mt-1">Сервер: 185.176.94.21:30120</p>
          {#if serverOnline}
            <p class="text-xs text-[#15ff00]/80 mt-1">Сервер онлайн — {serverPlayers}/{serverMaxPlayers}</p>
          {:else}
            <p class="text-xs text-[#f64a46]/80 mt-1">Сервер офлайн</p>
          {/if}
          {#if updateAvailable}
            <div class="mt-3 p-3 rounded-lg bg-[#f64a46]/10 border border-[#f64a46]/20">
              <p class="text-xs text-[#ff8c8c] mb-2">🔄 Доступно обновление до v{updateInfo.latest_version}</p>
              <button
                class="px-4 py-1.5 bg-[#f64a46]/20 hover:bg-[#f64a46]/30 rounded text-xs text-[#ff8c8c] transition-colors btn-ripple"
                onclick={openUpdateModal}
                onmouseenter={playHoverSound}
              >
                Скачать и установить
              </button>
            </div>
          {:else if updateChecked}
            <p class="text-xs text-green-400/70 mt-1">✓ Установлена последняя версия</p>
          {/if}
        </div>
      </div>

    {:else if activeMenu === "play"}
      <!-- ═══════════════════════════════════════
           СЕКЦИЯ: ИГРАТЬ (из imdex.tsx дизайн)
           ═══════════════════════════════════════ -->

      <!-- Оверлей анимации загрузки -->
      {#if launchPhase > 0}
        <div class="absolute inset-0 z-50 flex flex-col items-center justify-center bg-black/80 backdrop-blur-sm">
          <!-- ASTRA логотип -->
          <div class="mb-6 text-white tracking-wider" style="font-family: 'Armor Piercing 2.0 BB', sans-serif; text-shadow: 0 0 40px rgba(246,74,70,0.5); font-size: clamp(32px, 4.2vw, 40px);">
            ASTRA
          </div>

          <!-- Кольцевой прогресс -->
          <div class="relative w-24 h-24 mb-6">
            <svg class="w-24 h-24 -rotate-90" viewBox="0 0 100 100">
              <!-- Фоновое кольцо -->
              <circle cx="50" cy="50" r="42" stroke="rgba(255,255,255,0.08)" stroke-width="4" fill="none"/>
              <!-- Прогресс -->
              <circle cx="50" cy="50" r="42" stroke="#f64a46" stroke-width="4" fill="none"
                stroke-dasharray="{2 * Math.PI * 42}"
                stroke-dashoffset="{2 * Math.PI * 42 * (1 - launchProgress / 100)}"
                stroke-linecap="round"
                class="transition-all duration-500"
              />
            </svg>
            <!-- Процент в центре -->
            <div class="absolute inset-0 flex items-center justify-center">
              <span class="text-lg text-white/80" style="font-family: 'Proxima Nova Bold', sans-serif;">{launchProgress}%</span>
            </div>
          </div>

          <!-- Этапы загрузки -->
          <div class="flex flex-col gap-2 w-64">
            {#each launchPhases as phase, i}
              <div class="flex items-center gap-3 transition-all duration-300 {launchPhase > i ? 'opacity-100' : 'opacity-20'}">
                <!-- Индикатор этапа -->
                <div class="w-5 h-5 flex items-center justify-center flex-shrink-0">
                  {#if launchPhase > i + 1}
                    <!-- Завершён -->
                    <svg class="w-4 h-4 text-[#15ff00]" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3"><path d="M20 6L9 17l-5-5"/></svg>
                  {:else if launchPhase === i + 1}
                    <!-- Активный — спиннер -->
                    <svg class="w-4 h-4 text-[#f64a46] animate-spin" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
                      <circle cx="12" cy="12" r="9" stroke-dasharray="14 42" stroke-linecap="round"/>
                    </svg>
                  {:else}
                    <!-- Ожидание -->
                    <div class="w-1.5 h-1.5 bg-white/20 rounded-full"></div>
                  {/if}
                </div>
                <!-- Текст этапа -->
                <span class="text-xs {launchPhase === i + 1 ? 'text-white' : launchPhase > i + 1 ? 'text-white/40' : 'text-white/20'}"
                      style="font-family: 'Proxima Nova Semibold', sans-serif;">
                  {phase.label}
                </span>
              </div>
            {/each}
          </div>

          <!-- Красная полоска снизу -->
          <div class="mt-6 w-32 h-[2px] bg-gradient-to-r from-transparent via-[#f64a46] to-transparent animate-pulse"></div>
        </div>
      {/if}

      {#if fivemFound}
        <!-- #8 — Pill Play Button (из imdex.tsx: top-[528px] left-[750px] w-[210px] h-[70px] rounded-[1000px_0px_0px_1000px]) -->
        <!-- Декоративная подложка (gradient glow из imdex.tsx) -->
        <div class="absolute bottom-[42px] right-[0px] w-[365px] h-[70px] rounded-[1000px_0px_0px_1000px] pointer-events-none z-[29]
             bg-[linear-gradient(90deg,rgba(0,0,0,0)_0%,rgba(246,74,70,0.21)_100%)]"></div>
        <!-- Кнопка Играть — pill shape -->
        <button
          class="play-button absolute bottom-[42px] right-0 w-[210px] h-[70px] bg-[#f64a46] rounded-[1000px_0px_0px_1000px] cursor-pointer
                 hover:bg-[#ff5a56] active:scale-[0.98] transition-all duration-150
                 disabled:opacity-50 disabled:cursor-not-allowed z-30
                 {serverOnline && !isLaunching ? 'play-button-pulse' : ''}"
          style="{serverOnline && !isLaunching ? 'box-shadow: 0 0 0 0 rgba(246,74,70,0.4);' : ''}"
          onclick={handlePlay}
          onmouseenter={playHoverSound}
          disabled={isLaunching}
        >
          <!-- Прогресс-бар предзагрузки (фоновый) -->
          {#if precacheDownloading || precacheExtracting}
            <div class="absolute inset-0 bg-white/10 transition-all duration-300"
                 style="width: {precachePercent}%"></div>
          {/if}
          <div class="flex items-center justify-center gap-3 pl-6 relative z-10">
            <!-- Стрелка иконка -->
            {#if precacheDownloading}
              <svg class="w-4 h-4 animate-spin" viewBox="0 0 24 24" fill="none" stroke="white" stroke-width="2">
                <path d="M21 12a9 9 0 11-6.219-8.56"/>
              </svg>
            {:else if precacheExtracting}
              <svg class="w-4 h-4 animate-pulse" viewBox="0 0 24 24" fill="none" stroke="white" stroke-width="2">
                <rect x="3" y="3" width="18" height="18" rx="2"/>
                <path d="M9 12l2 2 4-4"/>
              </svg>
            {:else}
              <svg class="w-4 h-[17px]" viewBox="0 0 16 17" fill="white">
                <polygon points="0,0 16,8.5 0,17" />
              </svg>
            {/if}
            <span class="text-white text-[22px] tracking-[-0.44px] whitespace-nowrap"
                  style="font-family: 'Proxima Nova Semibold', sans-serif; font-weight: 600;">
              {playLabel}
            </span>
          </div>
        </button>
      {:else if isDownloading}
        <!-- Скачивание FiveM -->
        <div class="z-10 w-80 mt-4">
          <p class="text-sm text-gray-400 mb-3 text-center">
            {downloadSize || "Скачивание FiveM…"}
          </p>
          <div class="w-full h-2 bg-white/10 rounded-full overflow-hidden">
            <div
              class="h-full bg-gradient-to-r from-[#f64a46] to-[#ff8c4d] rounded-full transition-all duration-300"
              style="width: {downloadPercent}%"
            ></div>
          </div>
          <p class="text-xs text-gray-500 mt-2 text-center">{downloadPercent}%</p>
        </div>
      {:else}
        <!-- FiveM не найден -->
        <div class="flex items-center gap-2 mb-4 z-10">
          <span class="w-2 h-2 rounded-full bg-red-500"></span>
          <span class="text-sm text-red-400">FiveM не найден</span>
        </div>

        <button
          class="z-10 px-10 py-3 rounded-xl text-base font-bold tracking-wide
                 bg-[#f64a46] hover:bg-[#ff5a56] btn-ripple btn-bounce glow-hover
                 active:scale-95 transition-all duration-150"
          onclick={downloadAndInstall}
          onmouseenter={playHoverSound}
        >
          Скачать FiveM
        </button>

        <button
          class="z-10 mt-3 px-6 py-2 rounded-lg text-sm
                 bg-white/10 hover:bg-white/15 transition-colors btn-ripple"
          onclick={checkFivemAgain}
          onmouseenter={playHoverSound}
        >
          Проверить снова
        </button>

        {#if downloadError}
          <p class="mt-3 text-sm text-red-400 z-10">{downloadError}</p>
        {/if}
      {/if}

      <!-- Статус-сообщение -->
      {#if statusMessage}
        <p class="absolute bottom-20 left-1/2 -translate-x-1/2 text-sm text-gray-400 z-10">{statusMessage}</p>
      {/if}

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
  {#if onboardingActive}
    <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
    <div class="absolute inset-0 z-[500] flex items-center justify-center bg-[#0d0d0d]/95 backdrop-blur-md animate-onboarding-in">
      <div class="flex flex-col items-center justify-center w-full h-full max-w-md px-8 animate-onboarding-content">

        <!-- Шаг 1: Добро пожаловать -->
        {#if onboardingStep === 1}
          <div class="flex flex-col items-center text-center animate-onboarding-step">
            <!-- Логотип ASTRA -->
            <div class="mb-2 text-white tracking-[-1.2px] leading-none"
                 style="font-family: 'Armor Piercing 2.0 BB', 'Impact', sans-serif; text-shadow: 0 0 60px rgba(246,74,70,0.4); font-size: clamp(32px, 4.2vw, 40px);">
              ASTRA
            </div>
            <div class="w-[80px] h-[3px] bg-[#f64a46] rounded-full mb-8"></div>
            <h1 class="text-2xl text-white mb-3 text-reveal" style="font-family: 'Proxima Nova Bold', sans-serif; font-weight: 700; letter-spacing: -0.48px;">
              Добро пожаловать!
            </h1>
            <p class="text-sm text-white/40 mb-10 max-w-xs" style="font-family: 'Proxima Nova Semibold', sans-serif;">
              Лаунчер для подключения к ASTRA RP. Настроим всё за пару шагов.
            </p>
            <button
              class="px-10 py-3.5 bg-[#f64a46] hover:bg-[#ff5a56] active:scale-[0.97] rounded-xl text-base font-semibold transition-all duration-150 btn-ripple btn-bounce glow-hover"
              style="font-family: 'Proxima Nova Semibold', sans-serif; font-weight: 600;"
              onclick={onboardingNext}
              onmouseenter={playHoverSound}
            >
              Начать
            </button>
          </div>

        <!-- Шаг 2: Поиск FiveM -->
        {:else if onboardingStep === 2}
          <div class="flex flex-col items-center text-center animate-onboarding-step">
            <!-- Иконка поиска -->
            <div class="w-16 h-16 mb-6 flex items-center justify-center rounded-2xl bg-white/5 border border-white/10">
              {#if onboardingFivemSearching}
                <svg class="w-8 h-8 text-[#f64a46] animate-spin" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <circle cx="12" cy="12" r="9" stroke-dasharray="14 42" stroke-linecap="round"/>
                </svg>
              {:else if onboardingFivemResult === 'found'}
                <svg class="w-8 h-8 text-[#15ff00]" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><path d="M20 6L9 17l-5-5"/></svg>
              {:else}
                <svg class="w-8 h-8 text-white/30" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"/></svg>
              {/if}
            </div>

            <h2 class="text-xl text-white mb-2" style="font-family: 'Proxima Nova Bold', sans-serif; font-weight: 700; letter-spacing: -0.40px;">
              Поиск FiveM
            </h2>

            {#if onboardingFivemSearching}
              <p class="text-sm text-white/40 mb-8" style="font-family: 'Proxima Nova Semibold', sans-serif;">
                Ищем FiveM на вашем компьютере…
              </p>
            {:else if onboardingFivemResult === 'found'}
              <p class="text-sm text-[#15ff00]/80 mb-2" style="font-family: 'Proxima Nova Semibold', sans-serif;">
                ✓ FiveM найден автоматически
              </p>
              <p class="text-xs text-white/20 mb-8 truncate max-w-xs">{fivemPath}</p>
              <button
                class="px-10 py-3.5 bg-[#f64a46] hover:bg-[#ff5a56] active:scale-[0.97] rounded-xl text-base font-semibold transition-all duration-150 btn-ripple btn-bounce glow-hover"
                style="font-family: 'Proxima Nova Semibold', sans-serif; font-weight: 600;"
                onclick={onboardingNext}
                onmouseenter={playHoverSound}
              >
                Далее
              </button>
            {:else if onboardingFivemResult === 'not_found'}
              <p class="text-sm text-[#f64a46]/80 mb-6" style="font-family: 'Proxima Nova Semibold', sans-serif;">
                FiveM не найден на вашем компьютере
              </p>

              {#if isDownloading}
                <!-- Прогресс скачивания -->
                <div class="w-full max-w-xs mb-6">
                  <p class="text-xs text-white/40 mb-2 text-center">{downloadSize || "Скачивание FiveM…"}</p>
                  <div class="w-full h-2 bg-white/10 rounded-full overflow-hidden">
                    <div class="h-full bg-gradient-to-r from-[#f64a46] to-[#ff8c4d] rounded-full transition-all duration-300"
                         style="width: {downloadPercent}%"></div>
                  </div>
                  <p class="text-xs text-white/20 mt-1 text-center">{downloadPercent}%</p>
                </div>
              {:else}
                <!-- Кнопки: скачать / указать путь -->
                <div class="flex flex-col gap-3 w-full max-w-xs">
                  <button
                    class="w-full px-6 py-3 bg-[#f64a46] hover:bg-[#ff5a56] active:scale-[0.97] rounded-xl text-sm font-semibold transition-all duration-150 btn-ripple btn-bounce glow-hover"
                    style="font-family: 'Proxima Nova Semibold', sans-serif; font-weight: 600;"
                    onclick={onboardingDownloadFivem}
                    onmouseenter={playHoverSound}
                  >
                    Скачать FiveM
                  </button>
                  <button
                    class="w-full px-6 py-3 bg-white/5 hover:bg-white/10 rounded-xl text-sm text-white/50 hover:text-white/70 transition-all border border-white/5 btn-ripple"
                    style="font-family: 'Proxima Nova Semibold', sans-serif; font-weight: 600;"
                    onclick={onboardingSelectFivem}
                    onmouseenter={playHoverSound}
                  >
                    Указать путь вручную
                  </button>
                  <button
                    class="w-full px-6 py-2.5 bg-transparent hover:bg-white/5 rounded-xl text-xs text-white/30 hover:text-white/50 transition-all btn-ripple"
                    style="font-family: 'Proxima Nova Semibold', sans-serif;"
                    onclick={onboardingNext}
                    onmouseenter={playHoverSound}
                  >
                    Пропустить →
                  </button>
                </div>
              {/if}

              {#if downloadError}
                <p class="mt-3 text-xs text-red-400">{downloadError}</p>
              {/if}
            {:else}
              <p class="text-sm text-white/30 mb-8" style="font-family: 'Proxima Nova Semibold', sans-serif;">
                Проверяем установку…
              </p>
            {/if}
          </div>

        <!-- Шаг 3: Ввод никнейма -->
        {:else if onboardingStep === 3}
          <div class="flex flex-col items-center text-center animate-onboarding-step">
            <!-- Иконка пользователя -->
            <div class="w-16 h-16 mb-6 flex items-center justify-center rounded-2xl bg-white/5 border border-white/10">
              <svg class="w-8 h-8 text-white/50" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
                <path d="M20 21v-2a4 4 0 00-4-4H8a4 4 0 00-4 4v2"/>
                <circle cx="12" cy="7" r="4"/>
              </svg>
            </div>

            <h2 class="text-xl text-white mb-2" style="font-family: 'Proxima Nova Bold', sans-serif; font-weight: 700; letter-spacing: -0.40px;">
              Ваш никнейм
            </h2>
            <p class="text-sm text-white/40 mb-6" style="font-family: 'Proxima Nova Semibold', sans-serif;">
              Введите имя для отображения на сервере
            </p>

            <!-- Поле ввода никнейма -->
            <div class="w-full max-w-xs mb-8">
              <input
                type="text"
                class="w-full px-4 py-3 bg-white/5 border border-white/10 rounded-xl text-sm text-white placeholder-white/20 focus:outline-none focus:border-[#f64a46]/50 focus:bg-white/[0.07] transition-all"
                style="font-family: 'Proxima Nova Semibold', sans-serif; font-weight: 600;"
                placeholder="Ваш никнейм"
                bind:value={onboardingNickname}
                maxlength="20"
                onkeydown={(e) => { if (e.key === 'Enter') onboardingNext(); }}
              />
              <p class="text-[10px] text-white/20 mt-2">Можно изменить позже в настройках</p>
            </div>

            <button
              class="px-10 py-3.5 bg-[#f64a46] hover:bg-[#ff5a56] active:scale-[0.97] rounded-xl text-base font-semibold transition-all duration-150 disabled:opacity-50 disabled:cursor-not-allowed btn-ripple btn-bounce glow-hover"
              style="font-family: 'Proxima Nova Semibold', sans-serif; font-weight: 600;"
              onclick={onboardingNext}
              onmouseenter={playHoverSound}
              disabled={onboardingNicknameSaving}
            >
              {onboardingNicknameSaving ? "Сохранение…" : "Далее"}
            </button>
          </div>

        <!-- Шаг 4: Приятной игры! -->
        {:else if onboardingStep === 4}
          <div class="flex flex-col items-center text-center animate-onboarding-step">
            <!-- Иконка ракеты -->
            <div class="w-16 h-16 mb-6 flex items-center justify-center rounded-2xl bg-[#f64a46]/10 border border-[#f64a46]/20">
              <svg class="w-8 h-8 text-[#f64a46]" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
                <path d="M4.5 16.5c-1.5 1.26-2 5-2 5s3.74-.5 5-2c.71-.84.7-2.13-.09-2.91a2.18 2.18 0 00-2.91-.09z"/>
                <path d="M12 15l-3-3a22 22 0 012-3.95A12.88 12.88 0 0122 2c0 2.72-.78 7.5-6 11a22.35 22.35 0 01-4 2z"/>
                <path d="M9 12H4s.55-3.03 2-4c1.62-.91 3 0 3 0"/>
                <path d="M12 15v5s3.03-.55 4-2c.91-1.62 0-3 0-3"/>
              </svg>
            </div>

            <h2 class="text-2xl text-white mb-3" style="font-family: 'Proxima Nova Bold', sans-serif; font-weight: 700; letter-spacing: -0.48px;">
              Приятной игры!
            </h2>
            <p class="text-sm text-white/40 mb-10" style="font-family: 'Proxima Nova Semibold', sans-serif;">
              Всё настроено. Добро пожаловать на ASTRA RP!
            </p>

            <button
              class="px-10 py-3.5 bg-[#f64a46] hover:bg-[#ff5a56] active:scale-[0.97] rounded-xl text-base font-semibold transition-all duration-150 btn-ripple btn-bounce glow-hover"
              style="font-family: 'Proxima Nova Semibold', sans-serif; font-weight: 600;"
              onclick={onboardingNext}
              onmouseenter={playHoverSound}
            >
              Открыть лаунчер
            </button>
          </div>
        {/if}

        <!-- Индикатор шагов -->
        <div class="absolute bottom-8 left-1/2 -translate-x-1/2 flex items-center gap-2">
          {#each [1, 2, 3, 4] as step}
            <div class="w-2 h-2 rounded-full transition-all duration-300 {onboardingStep === step ? 'bg-[#f64a46] w-6' : onboardingStep > step ? 'bg-[#15ff00]/50' : 'bg-white/10'}"></div>
          {/each}
        </div>
      </div>
    </div>
  {/if}
</div>
