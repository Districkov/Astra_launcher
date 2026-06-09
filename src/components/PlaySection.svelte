<!--
  PlaySection.svelte — Секция «Играть» (кнопка, анимация запуска, скачивание FiveM)
-->
<script>
  let {
    launchPhase = 0,
    launchProgress = 0,
    launchPhases = [],
    fivemFound = false,
    isLaunching = false,
    serverOnline = false,
    offlineMode = false,
    isDownloading = false,
    downloadPercent = 0,
    downloadSize = "",
    downloadError = "",
    precacheDownloading = false,
    precacheExtracting = false,
    precachePercent = 0,
    playLabel = "Играть",
    statusMessage = "",
    handlePlay = async () => {},
    downloadAndInstall = async () => {},
    checkFivemAgain = async () => {},
    playHoverSound = () => {},
  } = $props();
</script>

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
        <circle cx="50" cy="50" r="42" stroke="rgba(255,255,255,0.08)" stroke-width="4" fill="none" />
        <!-- Прогресс -->
        <circle
          cx="50" cy="50" r="42"
          stroke="#f64a46" stroke-width="4" fill="none"
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
              <svg class="w-4 h-4 text-[#15ff00]" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3"><path d="M20 6L9 17l-5-5" /></svg>
            {:else if launchPhase === i + 1}
              <!-- Активный — спиннер -->
              <svg class="w-4 h-4 text-[#f64a46] animate-spin" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
                <circle cx="12" cy="12" r="9" stroke-dasharray="14 42" stroke-linecap="round" />
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

{#if offlineMode}
  <!-- Нет подключения к интернету -->
  <div class="z-10 text-center">
    <div class="mb-4 flex justify-center">
      <svg class="w-16 h-16 text-red-400/60" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
        <line x1="1" y1="1" x2="23" y2="23" />
        <path d="M16.72 11.06A10.94 10.94 0 0 1 19 12.55" />
        <path d="M5 12.55a10.94 10.94 0 0 1 5.17-2.39" />
        <path d="M10.71 5.05A16 16 0 0 1 22.56 9" />
        <path d="M1.42 9a15.91 15.91 0 0 1 4.7-2.88" />
        <path d="M8.53 16.11a6 6 0 0 1 6.95 0" />
        <line x1="12" y1="20" x2="12.01" y2="20" />
      </svg>
    </div>
    <h2 class="text-xl font-bold mb-2 text-red-400" style="font-family: 'Proxima Nova Bold', sans-serif;">Нет подключения</h2>
    <p class="text-sm text-gray-500">Проверьте подключение к интернету и перезапустите лаунчер</p>
  </div>
{:else if fivemFound}
  <!-- Декоративная подложка (gradient glow) -->
  <div class="absolute bottom-[42px] right-[0px] w-[365px] h-[70px] rounded-[1000px_0px_0px_1000px] pointer-events-none z-[29] bg-[linear-gradient(90deg,rgba(0,0,0,0)_0%,rgba(246,74,70,0.21)_100%)]"></div>
  <!-- Кнопка Играть — pill shape -->
  <button
    class="play-button absolute bottom-[42px] right-0 w-[210px] h-[70px] bg-[#f64a46] rounded-[1000px_0px_0px_1000px] cursor-pointer
           hover:bg-[#ff5a56] active:scale-[0.98] transition-all duration-150
           disabled:opacity-50 disabled:cursor-not-allowed z-30
           {serverOnline && !isLaunching ? 'play-button-pulse' : ''}"
    style="{serverOnline && !isLaunching ? 'box-shadow: 0 0 0 0 rgba(246,74,70,0.4);' : ''}"
    onclick={handlePlay}
    onmouseenter={playHoverSound}
    disabled={isLaunching || !serverOnline}
  >
    <!-- Прогресс-бар предзагрузки (фоновый) -->
    {#if precacheDownloading || precacheExtracting}
      <div class="absolute inset-0 bg-white/10 transition-all duration-300" style="width: {precachePercent}%"></div>
    {/if}
    <div class="flex items-center justify-center gap-3 pl-6 relative z-10">
      <!-- Стрелка иконка -->
      {#if precacheDownloading}
        <svg class="w-4 h-4 animate-spin" viewBox="0 0 24 24" fill="none" stroke="white" stroke-width="2">
          <path d="M21 12a9 9 0 11-6.219-8.56" />
        </svg>
      {:else if precacheExtracting}
        <svg class="w-4 h-4 animate-pulse" viewBox="0 0 24 24" fill="none" stroke="white" stroke-width="2">
          <rect x="3" y="3" width="18" height="18" rx="2" />
          <path d="M9 12l2 2 4-4" />
        </svg>
      {:else}
        <svg class="w-4 h-[17px]" viewBox="0 0 16 17" fill="white">
          <polygon points="0,0 16,8.5 0,17" />
        </svg>
      {/if}
      <span class="text-white text-[22px] tracking-[-0.44px] whitespace-nowrap"
            style="font-family: 'Proxima Nova Semibold', sans-serif; font-weight: 600;">
        {!serverOnline ? "Сервер офлайн" : playLabel}
      </span>
    </div>
  </button>
  {#if !serverOnline && !isLaunching}
    <p class="absolute bottom-14 right-4 text-xs text-gray-500 z-10">Сервер временно недоступен</p>
  {/if}
{:else if isDownloading}
  <!-- Скачивание FiveM -->
  <div class="z-10 w-80 mt-4">
    <p class="text-sm text-gray-400 mb-3 text-center">
      {downloadSize || "Скачивание FiveM…"}
    </p>
    <div class="w-full h-2 bg-white/10 rounded-full overflow-hidden">
      <div class="h-full bg-gradient-to-r from-[#f64a46] to-[#ff8c4d] rounded-full transition-all duration-300"
           style="width: {downloadPercent}%"></div>
    </div>
    <p class="text-xs text-gray-500 mt-2 text-center">{downloadPercent}%</p>
    {#if downloadPercent >= 100}
      <button
        class="mt-3 w-full px-6 py-2 rounded-lg text-sm font-medium
               bg-white/10 hover:bg-white/15 transition-colors btn-ripple"
        onclick={checkFivemAgain}
        onmouseenter={playHoverSound}
      >
        Проверить установку
      </button>
    {/if}
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

<style>
  :global(.play-button-pulse) {
    animation: pulse-button 2s infinite;
  }

  @keyframes pulse-button {
    0% {
      box-shadow: 0 0 0 0 rgba(246, 74, 70, 0.4);
    }
    50% {
      box-shadow: 0 0 0 10px rgba(246, 74, 70, 0);
    }
    100% {
      box-shadow: 0 0 0 0 rgba(246, 74, 70, 0);
    }
  }
</style>
