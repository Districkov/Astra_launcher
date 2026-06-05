<!--
  WindowControls.svelte — Кнопки управления окном (свернуть / закрыть)
  Принимает onClose, onMinimize, playHoverSound, playClickSound
-->
<script>
  import { invoke } from "@tauri-apps/api/core";

  let { onClose = () => {}, playHoverSound = () => {} } = $props();

  function minimize() {
    invoke("window_minimize");
  }
</script>

<div class="absolute top-0 right-0 z-[100] flex items-center gap-0">
  <button
    class="w-10 h-8 flex items-center justify-center text-white/40 hover:text-white hover:bg-white/10 transition-colors"
    onclick={minimize}
    onmouseenter={playHoverSound}
    title="Свернуть"
  >
    <svg width="12" height="1" viewBox="0 0 12 1" fill="currentColor"><rect width="12" height="1"/></svg>
  </button>
  <button
    class="w-10 h-8 flex items-center justify-center text-white/40 hover:text-[#f64a46] hover:bg-white/10 transition-colors"
    onclick={onClose}
    onmouseenter={playHoverSound}
    title="Закрыть"
  >
    <svg width="10" height="10" viewBox="0 0 10 10" fill="none" stroke="currentColor" stroke-width="1.2"><line x1="1" y1="1" x2="9" y2="9"/><line x1="9" y1="1" x2="1" y2="9"/></svg>
  </button>
</div>
