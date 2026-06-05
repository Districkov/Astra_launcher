<!--
  Settings.svelte — Секция настроек (путь к FiveM, информация о лаунчере)
-->
<script>
  let {
    fivemPath = "",
    fivemFound = false,
    updateInfo = {},
    updateAvailable = false,
    updateChecked = false,
    serverOnline = false,
    serverPlayers = 0,
    serverMaxPlayers = 0,
    selectFivemPath = async () => {},
    autoFindFivem = async () => {},
    openUpdateModal = () => {},
    playHoverSound = () => {},
  } = $props();
</script>

<div class="w-full max-w-lg px-8 z-10">
  <h2 class="text-2xl mb-6 text-reveal" style="font-family: 'Proxima Nova Bold', sans-serif; font-weight: 700; letter-spacing: -0.48px;">
    Настройки
  </h2>

  <!-- Путь к FiveM -->
  <div class="settings-card bg-white/5 rounded-lg p-5 mb-4 stagger-1">
    <h3 class="text-sm font-semibold text-white/70 mb-3" style="font-family: 'Proxima Nova Semibold', sans-serif;">
      Путь к FiveM
    </h3>
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
    <h3 class="text-sm font-semibold text-white/70 mb-3" style="font-family: 'Proxima Nova Semibold', sans-serif;">
      О лаунчере
    </h3>
    <p class="text-xs text-white/60">ASTRA Launcher v{updateInfo.current_version || '...'}</p>
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

<style>
  :global(.settings-card) {
    transition: all 0.3s ease;
  }

  :global(.settings-card:hover) {
    background: rgba(255, 255, 255, 0.08);
  }
</style>
