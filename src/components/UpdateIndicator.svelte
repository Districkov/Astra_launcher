<!--
  UpdateIndicator.svelte — Индикатор доступного обновления
  Принимает updateAvailable, updateInfo, onOpen (callback)
-->
<script>
  let { updateAvailable = false, updateInfo = {}, onOpen = () => {}, playHoverSound = () => {} } = $props();
</script>

{#if updateAvailable}
  <button
    class="absolute top-[40px] left-[170px] z-[80] flex items-center gap-2 px-3 py-1.5 rounded-lg bg-[#f64a46]/15 border border-[#f64a46]/20 cursor-pointer hover:bg-[#f64a46]/25 transition-colors"
    onclick={onOpen}
    onmouseenter={playHoverSound}
    aria-label="Доступно обновление"
  >
    <div class="w-2 h-2 rounded-full bg-[#f64a46] update-badge-dot"></div>
    <span class="text-xs text-[#ff8c8c]">Обновление v{updateInfo.latest_version}</span>
  </button>
{/if}
