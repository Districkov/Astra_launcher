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
  import { open as openUrl } from "@tauri-apps/plugin-shell";
  import { onMount } from "svelte";
  import { TIMING } from "./constants";
  import { getServerStore } from "./stores/serverStore.svelte";
  import { getUpdateStore } from "./stores/updateStore.svelte";
  import { getFivemStore } from "./stores/fivemStore.svelte";
  import { getAudioStore } from "./stores/audioStore.svelte";
  import { getOnboardingStore } from "./stores/onboardingStore.svelte";
  import { getStoreStore } from "./stores/storeStore.svelte";
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
  import StoreSection from "./components/StoreSection.svelte";
  import OnboardingModal from "./components/OnboardingModal.svelte";

  // ── Store instances ────────────────────────────
  const server = getServerStore();
  const update = getUpdateStore();
  const fivem = getFivemStore();
  const audio = getAudioStore();
  const onboarding = getOnboardingStore();
  const store = getStoreStore();

  let appWindow;
  try {
    appWindow = getCurrentWindow();
  } catch (e) {
    // window not available
    appWindow = null;
  }

  // ── Локальное UI-состояние (не в stores) ────────
  let activeMenu = $state("play");
  let previousMenu = $state("play");
  let showCloseConfirm = $state(false);
  let showLogoutConfirm = $state(false);
  let pageTransition = $state("");
  let launcherReady = $state(false);
  let browserActive = $state(false);
  let username = $state("Player");
  let discordRpcActive = $state(false);

  // Параллакс — смещение фона при движении мыши (throttled via rAF)
  let parallaxX = $state(0);
  let parallaxY = $state(0);
  let parallaxRafId = 0; // not reactive — plain variable for rAF throttling

  // DPI масштаб для ASTRA текста на 2K/4K
  let dpiScale = $state(1);

  function handleMouseMove(e) {
    if (parallaxRafId) return; // уже запланирован кадр
    parallaxRafId = requestAnimationFrame(() => {
      const cx = window.innerWidth / 2;
      const cy = window.innerHeight / 2;
      parallaxX = ((e.clientX - cx) / cx) * 12;
      parallaxY = ((e.clientY - cy) / cy) * 8;
      parallaxRafId = 0;
    });
  }

  // Клавиатурная навигация — Escape закрывает модалки
  function handleKeyDown(e) {
    if (e.key === "Escape") {
      if (showCloseConfirm) { showCloseConfirm = false; return; }
      if (showLogoutConfirm) { showLogoutConfirm = false; return; }
      if (update.showModal) { update.closeUpdateModal(audio.playClickSound); return; }
      if (onboarding.active) { onboarding.onboardingBack(); return; }
    }
  }

  // Частицы — позиции кэшируются один раз (20 вместо 30 для производительности)
  let particleStyles = $state([]);
  function initParticles() {
    particleStyles = Array.from({ length: 20 }, () => ({
      left: `${Math.random() * 100}%`,
      top: `${Math.random() * 100}%`,
      delay: `${Math.random() * 8}s`,
      duration: `${4 + Math.random() * 6}s`,
      width: `${1 + Math.random() * 2}px`,
      height: `${1 + Math.random() * 2}px`,
      willChange: "transform, opacity",
    }));
  }



  // Производное: текст кнопки
  let playLabel = $derived(
    fivem.precacheDownloading ? `Скачивание кеша ${fivem.precachePercent}%${fivem.precacheTotal ? ' (' + fivem.precacheDownloaded + '/' + fivem.precacheTotal + ')' : ''}` :
    fivem.precacheExtracting ? `Распаковка кеша ${fivem.precachePercent}%` :
    fivem.launchPhase > 0 ? fivem.launchPhases[fivem.launchPhase - 1]?.label || "Запуск…" :
    fivem.launching ? "Запуск…" : "Играть"
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

  // ── Делегированные функции (stores) ─────────────
  const addNotification = server.addNotification;

  // ── #9 Имя пользователя ────────────────────────
  async function loadUsername() {
    try {
      const name = await invoke("get_username");
      username = name || "Player";
    } catch (e) {
      username = "Player";
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

  // ── Telegram в лаунчере (встроенный webview) ──
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
    audio.playClickSound();
    if (id === "discord") {
      openUrl("https://discord.gg/zfVss2mNVW");
      return;
    }
    if (id === "news") {
      openEmbeddedBrowser("https://t.me/astrarp5");
      return;
    }
    if (id === "forum") {
      openUrl("https://forum.astra-rp.fun");
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
    // Reset transition after animation completes
    setTimeout(() => { pageTransition = ""; }, TIMING.transitionDuration);

    if (id === "settings") {
      fivem.loadFivemPath();
    }
  }

  // ── Инициализация ───────────────────────────────
  onMount(() => {
    // Вычисляем DPI масштаб на основе device pixel ratio
    dpiScale = window.devicePixelRatio || 1;

    // Получаем версию приложения из Tauri
    getVersion().then(v => { update.setCurrentVersion(v); }).catch(() => {});

    initParticles();
    setTimeout(() => { launcherReady = true; }, TIMING.launcherReadyDelay);

    // #11 — Инициализация звуков через AudioContext
    audio.initAudio();

    server.loadServerStatus();
    fivem.loadFivemPath();
    fivem.autoFindFivem(); // No sound on auto-search at startup
    loadUsername();
    server.loadServerPing();
    update.checkUpdates();
    initDiscordRpc();
    onboarding.checkOnboarding();

    const serverInterval = setInterval(() => server.loadServerStatus(), TIMING.serverStatusInterval);
    const pingInterval = setInterval(() => server.loadServerPing(), TIMING.serverPingInterval);
    // Проверка обновлений каждые 30 мин
    const updateInterval = setInterval(() => update.checkUpdates(), TIMING.updateCheckInterval);

    return () => {
      clearInterval(serverInterval);
      clearInterval(pingInterval);
      clearInterval(updateInterval);
    };
  });
</script>

<!-- ── Корневой контейнер ── -->
<svelte:window onmousemove={handleMouseMove} onkeydown={handleKeyDown} oncontextmenu={(e) => e.preventDefault()} />

<div class="astra-cursor relative w-full h-full min-w-[960px] min-h-[600px] rounded-[5px] overflow-hidden bg-[#0d0d0d] text-white select-none transition-opacity duration-700 {launcherReady ? 'opacity-100' : 'opacity-0'}">

  <!-- ═══════════════════════════════════════════════
       #6 АНИМИРОВАННЫЙ ФОН (частицы + градиент)
       ═══════════════════════════════════════════════ -->
  <Background {parallaxX} {parallaxY} {dpiScale} {particleStyles} />

  <!-- ═══════════════════════════════════════════════
       #7 УВЕДОМЛЕНИЯ (toast)
       ═══════════════════════════════════════════════ -->
  <Notifications notifications={server.notifications} />

  <!-- ═══════════════════════════════════════════════
       #5 ИНДИКАТОР ОБНОВЛЕНИЯ
       ═══════════════════════════════════════════════ -->
  <UpdateIndicator updateAvailable={update.available} updateInfo={update.info} onOpen={() => update.openUpdateModal(audio.playClickSound)} playHoverSound={audio.playHoverSound} />

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
  <WindowControls onClose={() => { audio.playClickSound(); showCloseConfirm = true; }} playHoverSound={audio.playHoverSound} />

  <!-- ═══════════════════════════════════════════════
       МОДАЛКА ПОДТВЕРЖДЕНИЯ ЗАКРЫТИЯ
       ═══════════════════════════════════════════════ -->
  <CloseConfirmModal show={showCloseConfirm} onClose={() => { showCloseConfirm = false; }} playClickSound={audio.playClickSound} playHoverSound={audio.playHoverSound} />

  <!-- ═══════════════════════════════════════════════
       МОДАЛКА ПОДТВЕРЖДЕНИЯ ВЫХОДА ИЗ ПРОФИЛЯ
       ═══════════════════════════════════════════════ -->
  <LogoutConfirmModal
    show={showLogoutConfirm}
    onClose={() => { showLogoutConfirm = false; }}
    onLogout={() => { username = "Player"; server.addNotification("Вы вышли из аккаунта", "info"); }}
    playClickSound={audio.playClickSound}
    playHoverSound={audio.playHoverSound}
  />

  <!-- ═══════════════════════════════════════════════
       МОДАЛКА ОБНОВЛЕНИЯ
       ═══════════════════════════════════════════════ -->
  <UpdateModal
    show={update.showModal}
    updateInfo={update.info}
    updateDownloading={update.downloading}
    updateDownloadPercent={update.downloadPercent}
    updateDownloaded={update.downloaded}
    updateTotal={update.total}
    updateError={update.error}
    onStartDownload={() => update.startUpdateDownload(audio.playClickSound)}
    onClose={() => update.closeUpdateModal(audio.playClickSound)}
    playClickSound={audio.playClickSound}
    playHoverSound={audio.playHoverSound}
  />





  <!-- ═══════════════════════════════════════════════
       БОКОВАЯ ПАНЕЛЬ (Sidebar) — из imdex.tsx
       Фон: #1b1b1b, ширина: 157px
       ═══════════════════════════════════════════════ -->
  <Sidebar
    {activeMenu}
    serverOnline={server.online}
    serverLoading={server.loading}
    serverPlayers={server.players}
    serverPing={server.ping}
    {username}
    {menuItems}
    {selectMenu}
    playHoverSound={audio.playHoverSound}
    playClickSound={audio.playClickSound}
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
        fivemPath={fivem.path}
        fivemFound={fivem.found}
        updateInfo={update.info}
        updateAvailable={update.available}
        updateChecked={update.checked}
        updateChecking={update.checking}
        serverOnline={server.online}
        serverPlayers={server.players}
        serverMaxPlayers={server.maxPlayers}
        serverLoading={server.loading}
        selectFivemPath={() => fivem.selectFivemPath(audio.playClickSound)}
        autoFindFivem={() => fivem.autoFindFivem(audio.playClickSound)}
        openUpdateModal={() => update.openUpdateModal(audio.playClickSound)}
        checkUpdates={() => update.checkUpdates()}
        refreshServerStatus={() => server.refreshServerStatus()}
        playHoverSound={audio.playHoverSound}
      />

    {:else if activeMenu === "play"}
      <!-- ═══════════════════════════════════════
           СЕКЦИЯ: ИГРАТЬ (из imdex.tsx дизайн)
           ═══════════════════════════════════════ -->
      <PlaySection
        launchPhase={fivem.launchPhase}
        launchProgress={fivem.launchProgress}
        launchPhases={fivem.launchPhases}
        fivemFound={fivem.found}
        isLaunching={fivem.launching}
        serverOnline={server.online}
        offlineMode={server.offline}
        isDownloading={fivem.downloading}
        downloadPercent={fivem.downloadPercent}
        downloadSize={fivem.downloadSize}
        downloadError={fivem.downloadError}
        precacheDownloading={fivem.precacheDownloading}
        precacheExtracting={fivem.precacheExtracting}
        precachePercent={fivem.precachePercent}
        {playLabel}
        statusMessage={fivem.statusMessage}
        handlePlay={() => fivem.handlePlay(audio.playClickSound, server.online, setDiscordRpcPlaying)}
        downloadAndInstall={() => fivem.downloadAndInstall(audio.playClickSound)}
        playHoverSound={audio.playHoverSound}
      />

    {:else if activeMenu === "store"}
      <!-- ═══════════════════════════════════════
           СЕКЦИЯ: МАГАЗИН
           ═══════════════════════════════════════ -->
      <StoreSection
        {store}
        playClickSound={audio.playClickSound}
        playHoverSound={audio.playHoverSound}
        openEmbeddedBrowser={openEmbeddedBrowser}
      />

    {:else}
      <!-- ═══════════════════════════════════════
           ЗАГЛУШКА (Форум/Discord)
           ═══════════════════════════════════════ -->
      <div class="z-10 text-center">
        <div class="mb-4 flex justify-center">
          {#if activeMenu === 'forum'}
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
    onboardingActive={onboarding.active}
    onboardingStep={onboarding.step}
    onboardingFivemSearching={onboarding.fivemSearching}
    onboardingFivemResult={onboarding.fivemResult}
    onboardingNickname={onboarding.nickname}
    onboardingNicknameSaving={onboarding.nicknameSaving}
    fivemPath={fivem.path}
    isDownloading={fivem.downloading}
    downloadPercent={fivem.downloadPercent}
    downloadSize={fivem.downloadSize}
    downloadError={fivem.downloadError}
    statusMessage={fivem.statusMessage}
    onboardingNext={() => onboarding.onboardingNext(audio.playClickSound, { value: username }, (v) => { username = v; })}
    onboardingSelectFivem={() => onboarding.onboardingSelectFivem(audio.playClickSound)}
    onboardingDownloadFivem={() => onboarding.onboardingDownloadFivem(audio.playClickSound)}
    playHoverSound={audio.playHoverSound}
    onNicknameChange={(val) => { onboarding.nickname = val; }}
  />
</div>
