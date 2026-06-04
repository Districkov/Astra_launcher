<script>
  export let menuItems = [];
  export let activeMenu = "play";
  export let serverOnline = false;
  export let serverPlayers = 0;
  export let serverMaxPlayers = 0;
  export let serverLoading = false;
  export let fivemFound = false;
  export let isLaunching = false;
  export let playLabel = "Играть";
  export let statusMessage = "";
  export let cacheInfo = { path: "", size_mb: 0, file_count: 0 };
  export let cacheLoading = false;

  export let handlePlay = () => {};
  export let downloadAndInstall = () => {};
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";

  async function openEmbeddedBrowser(url) {
    try {
      await invoke("create_embedded_webview", { url });
    } catch (err) {
      console.error("create_embedded_webview error:", err);
      window.open(url, "_blank");
    }
  }

  export let selectMenu = () => {};
  export let selectFivemPath = () => {};
  export let clearCache = () => {};

  import GTAV334 from "../assets/GTAV (33) 4.png";
  import star23 from "../assets/Star 23.svg";
  import star24 from "../assets/Star 24.svg";
  import subtract from "../assets/Subtract.svg";
  import polygon13 from "../assets/Polygon 13.svg";
  import vector6 from "../assets/Subtract1.svg";
  import image from "../assets/players.svg";

  $: serverLoadedText = serverLoading
    ? "Загрузка…"
    : serverOnline
    ? `${serverPlayers}/${serverMaxPlayers}`
    : "Офлайн";
</script>

<div class="relative w-full h-full bg-[#0d0d0d]">
  <!-- Background with decorative elements -->
  <div class="absolute inset-0 pointer-events-none overflow-hidden">
    <img class="absolute top-[10%] right-[20%] w-[300px] h-[250px]" alt="" src={star23} aria-hidden="true" />
    <img class="absolute top-0 left-[10%] w-[350px] h-[200px]" alt="" src={star24} aria-hidden="true" />
    <img class="absolute top-0 right-0 w-[50%] h-[70%] object-cover" alt="" src={subtract} aria-hidden="true" />
    <img class="absolute top-[10%] right-[5%] w-[55%] h-[75%] object-cover" alt="" src={GTAV334} aria-hidden="true" />
    <img class="absolute bottom-0 left-0 w-[65%] h-[55%]" alt="" src={image} aria-hidden="true" />
  </div>

  <!-- Sidebar Panel -->
  <aside class="absolute left-0 top-0 w-40 h-full bg-[#1b1b1b] border-r border-white/10 flex flex-col z-20 overflow-y-auto">
    <!-- Logo -->
    <div class="px-4 py-5 flex-shrink-0">
    <div class="text-2xl font-bold tracking-wider text-white [text-shadow:0px_0px_100px_#ffffff]" style="font-family: 'Armor Piercing 2.0 BB', 'Impact', sans-serif; font-size: clamp(32px, 4.2vw, 40px);">ASTRA</div>
      <div class="w-14 h-0.5 bg-[#f64a46] rounded-full mt-2"></div>
    </div>

    <!-- Online Status -->
    <div class="px-4 py-2 flex items-center gap-1.5 flex-shrink-0">
      <div class="flex items-center gap-1">
        <span class="w-2 h-2 rounded-full {serverOnline ? 'bg-green-500' : 'bg-red-500'}"></span>
        <span class="w-1 h-1 rounded-full bg-green-400 {serverOnline && !serverLoading ? 'animate-pulse' : 'hidden'}"></span>
      </div>
      <span class="text-[11px] text-gray-400">Онлайн: <span class="text-white font-semibold text-xs">{serverLoadedText}</span></span>
    </div>

    <div class="mx-3 my-2 h-px bg-white/10"></div>

    <!-- Navigation Menu -->
    <nav class="flex-1 px-2 py-2 space-y-1 overflow-y-auto">
      {#each menuItems as item}
        <button
          type="button"
          aria-current={activeMenu === item.id ? "page" : undefined}
          class={"all-unset w-full flex items-center gap-2.5 px-3 py-2.5 rounded text-xs transition-all cursor-pointer " +
            (activeMenu === item.id ? "bg-[#f64a46]/25 text-white font-medium" : "text-[#999] hover:bg-white/5 hover:text-gray-200")}
          on:click={() => {
            if (item.id === 'discord') {
              window.open('https://discord.gg/zfVss2mNVW', '_blank');
              return;
            }
            if (item.id === 'news') {
              openEmbeddedBrowser('https://t.me/astrarp5');
              return;
            }
            selectMenu(item.id);
          }}
        >
          <img src={item.icon} alt="" class="w-3 h-3 opacity-90" />
          <span class="truncate">{item.label}</span>
        </button>
      {/each}
    </nav>

    <div class="mx-3 my-2 h-px bg-white/10"></div>

    <!-- User Profile -->
    <div class="px-3 py-3 flex items-center gap-2 flex-shrink-0">
      <div class="w-7 h-7 rounded-full bg-gradient-to-br from-red-500 to-orange-600 flex items-center justify-center flex-shrink-0 text-xs font-bold text-white">
        R
      </div>
      <div class="flex-1 min-w-0">
        <p class="text-xs font-medium text-white truncate">r4kuzan</p>
        <p class="text-[10px] text-gray-500 truncate">ID: 10042</p>
      </div>
    </div>

    <button type="button" class="all-unset w-full px-3 py-2 text-xs text-gray-400 hover:text-gray-300 transition-colors text-left cursor-pointer flex-shrink-0" on:click={() => selectMenu('settings')}>
      Выйти
    </button>
  </aside>

  <!-- Main Content Area -->
  <main class="relative ml-40 w-[calc(100%-160px)] h-full flex flex-col items-center justify-center z-10">
    {#if activeMenu === 'play'}
      <!-- Play Section -->
      <div class="text-center space-y-6">
        <h2 class="text-6xl font-black tracking-wider text-white drop-shadow-2xl" style="font-family: 'Armor Piercing 2.0 BB', 'Impact', sans-serif; text-shadow: 0 0 60px rgba(255,255,255,0.3)">ASTRA</h2>

        {#if fivemFound}
          <div class="flex items-center gap-2 justify-center">
            <span class="w-2.5 h-2.5 rounded-full bg-green-500 animate-pulse"></span>
            <span class="text-sm text-green-400">FiveM готов к запуску</span>
          </div>

          <button
            type="button"
            class="mt-12 px-24 py-5 rounded-3xl text-3xl font-bold tracking-widest text-white bg-[#f64a46] hover:bg-[#ff6666] active:scale-95 transition-all shadow-2xl disabled:opacity-50 disabled:cursor-not-allowed"
            on:click={handlePlay}
            disabled={isLaunching}
          >
            {playLabel}
          </button>
        {:else}
          <div class="flex items-center gap-2 justify-center mt-6">
            <span class="w-2.5 h-2.5 rounded-full bg-red-500"></span>
            <span class="text-sm text-red-400">FiveM не установлен</span>
          </div>

          <button type="button" class="mt-12 px-16 py-4 rounded-3xl text-xl font-bold tracking-wide text-white bg-[#f64a46] hover:bg-[#ff6666] active:scale-95 transition-all shadow-2xl" on:click={downloadAndInstall}>
            Скачать FiveM
          </button>
        {/if}

        {#if statusMessage}
          <p class="text-sm text-gray-300 mt-6 animate-pulse">{statusMessage}</p>
        {/if}
      </div>

    {:else if activeMenu === 'settings'}
      <!-- Settings Section -->
      <div class="w-full h-full flex items-center justify-center p-8">
        <div class="w-full max-w-3xl bg-black/40 backdrop-blur-xl rounded-3xl border border-white/10 p-8 space-y-6 max-h-[90vh] overflow-y-auto">
          <h2 class="text-4xl font-bold text-white mb-6">Настройки</h2>

          <div class="bg-white/5 backdrop-blur-sm rounded-2xl border border-white/10 p-6">
            <h3 class="text-lg font-semibold text-white mb-4">🎮 FiveM</h3>
            <div class="flex gap-3 mb-4">
              <div class="flex-1 bg-black/50 rounded-lg px-4 py-2.5 text-xs text-gray-300 truncate border border-white/5">
                {fivemFound ? "✓ Установлен" : "✗ Не найден"}
              </div>
              <button class="px-6 py-2.5 bg-[#f64a46] hover:bg-[#ff6666] rounded-lg text-xs font-semibold text-white transition-colors" on:click={selectFivemPath}>
                Выбрать
              </button>
            </div>
            <p class="text-xs text-gray-500 break-all">{cacheInfo.path || "Путь не указан"}</p>
          </div>

          <div class="bg-white/5 backdrop-blur-sm rounded-2xl border border-white/10 p-6">
            <h3 class="text-lg font-semibold text-white mb-4">🗄️ Кэш сервера</h3>
            <div class="grid grid-cols-2 gap-4 mb-4">
              <div>
                <p class="text-xs text-gray-500 mb-1">Размер</p>
                <p class="text-lg text-white font-bold">{cacheInfo.size_mb} МБ</p>
              </div>
              <div>
                <p class="text-xs text-gray-500 mb-1">Файлов</p>
                <p class="text-lg text-white font-bold">{cacheInfo.file_count}</p>
              </div>
            </div>
            <button class="w-full px-4 py-2.5 bg-red-600/70 hover:bg-red-600 rounded-lg text-xs font-semibold text-white transition-colors disabled:opacity-40" on:click={clearCache} disabled={cacheLoading || cacheInfo.size_mb === 0}>
              Очистить кэш
            </button>
          </div>

          <div class="bg-white/5 backdrop-blur-sm rounded-2xl border border-white/10 p-6">
            <h3 class="text-lg font-semibold text-white mb-3">ℹ️ О лаунчере</h3>
            <p class="text-xs text-gray-400">ASTRA Launcher v1.0</p>
            <p class="text-xs text-gray-400 mt-2">Сервер: 185.176.94.21:30120</p>
            <p class="text-xs {serverOnline ? 'text-green-400' : 'text-red-400'} mt-2 font-medium">
              {serverOnline ? `Статус: Онлайн ${serverPlayers}/${serverMaxPlayers}` : "Статус: Сервер офлайн"}
            </p>
          </div>
        </div>
      </div>

    {:else}
      <!-- Other Sections (Placeholder) -->
      <div class="text-center space-y-4">
        <p class="text-7xl opacity-80">{#if activeMenu === 'store'}🛒{:else if activeMenu === 'forum'}💬{:else}📋{/if}</p>
        <h3 class="text-3xl font-bold text-white">{menuItems.find(m => m.id === activeMenu)?.label || "Раздел"}</h3>
        <p class="text-gray-500">Раздел в разработке</p>
      </div>
    {/if}
  </main>
</div>

<style>
  .all-unset {
    all: unset;
  }
</style>
