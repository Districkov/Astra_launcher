<!--
  LogoutConfirmModal.svelte — Модалка подтверждения выхода из профиля
-->
<script>
  import { invoke } from "@tauri-apps/api/core";

  let { show = false, onClose = () => {}, onLogout = () => {}, playClickSound = () => {}, playHoverSound = () => {} } = $props();

  function doLogout() {
    playClickSound();
    invoke("save_username", { name: "Player" });
    onLogout();
    onClose();
  }
</script>

{#if show}
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="absolute inset-0 z-[300] flex items-center justify-center bg-black/70 backdrop-blur-sm animate-fade-in" role="presentation" onclick={onClose}>
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div role="dialog" tabindex="0" aria-labelledby="logout-dialog-title" aria-modal="true" class="bg-[#1b1b1b] border border-white/10 rounded-xl px-8 py-6 max-w-xs w-full shadow-2xl" onclick={(e) => e.stopPropagation()} onkeydown={(e) => { if (e.key === "Escape") onClose(); }}>
      <!-- Логотип ASTRA + красная полоска -->
      <div class="text-center mb-5">
        <div class="text-white tracking-[-0.8px] leading-none" style="font-family: 'Armor Piercing 2.0 BB', 'Impact', sans-serif; font-size: 28px;">
          ASTRA
        </div>
        <div class="mt-1.5 mx-auto w-[50px] h-[3px] bg-[#f64a46] rounded-full"></div>
      </div>
      <h3 id="logout-dialog-title" class="text-lg text-white mb-1 text-center" style="font-family: 'Proxima Nova Bold', sans-serif; font-weight: 700; letter-spacing: -0.36px;">
        Выйти из аккаунта?
      </h3>
      <p class="text-sm text-white/30 mb-5 text-center" style="font-family: 'Proxima Nova Semibold', sans-serif;">
        Имя будет сброшено до «Player»
      </p>
      <div class="flex gap-3">
        <button
          class="flex-1 px-4 py-2.5 rounded-lg bg-[#f64a46] hover:bg-[#ff5a56] active:scale-[0.97] text-sm text-white font-medium transition-all btn-ripple btn-bounce glow-hover"
          style="font-family: 'Proxima Nova Semibold', sans-serif; font-weight: 600;"
          onclick={(e) => { e.stopPropagation(); doLogout(); }}
          onmouseenter={playHoverSound}
        >
          Выйти
        </button>
        <button
          class="flex-1 px-4 py-2.5 rounded-lg bg-white/5 hover:bg-white/10 text-sm text-white/50 hover:text-white/70 font-medium transition-all border border-white/5 btn-ripple"
          style="font-family: 'Proxima Nova Semibold', sans-serif; font-weight: 600;"
          onclick={(e) => { e.stopPropagation(); playClickSound(); onClose(); }}
          onmouseenter={playHoverSound}
        >
          Отмена
        </button>
      </div>
    </div>
  </div>
{/if}
