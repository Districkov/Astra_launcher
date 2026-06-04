<script lang="ts">
  import type { UpdateInfo } from "../types";

  export let showUpdateModal: boolean;
  export let updateInfo: UpdateInfo;
  export let updateDownloading: boolean;
  export let updateDownloadPercent: number;
  export let updateDownloaded: number;
  export let updateTotal: number;
  export let updateError: string;

  export let closeUpdateModal: () => void;
  export let handleInstallUpdate: () => Promise<void>;
  export let playHoverSound: () => void;
</script>

{#if showUpdateModal}
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="absolute inset-0 z-[300] flex items-center justify-center bg-black/70 backdrop-blur-sm animate-fade-in"
    role="presentation"
    onclick={closeUpdateModal}
  >
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div
      role="dialog"
      tabindex="0"
      aria-labelledby="update-dialog-title"
      aria-modal="true"
      class="bg-[#1b1b1b] border border-white/10 rounded-xl px-8 py-6 max-w-md w-full shadow-2xl"
      onclick={(e) => e.stopPropagation()}
      onkeydown={(e) => {
        if (e.key === "Escape") closeUpdateModal();
      }}
    >
      <!-- Логотип ASTRA -->
      <div class="text-center mb-4">
        <div
          class="text-white tracking-[-0.8px] leading-none"
          style="font-family: 'Armor Piercing 2.0 BB', 'Impact', sans-serif; font-size: 28px;"
        >
          ASTRA
        </div>
        <div class="mt-1.5 mx-auto w-[50px] h-[3px] bg-[#f64a46] rounded-full"></div>
      </div>

      <h3
        id="update-dialog-title"
        class="text-lg text-white mb-1 text-center"
        style="font-family: 'Proxima Nova Bold', sans-serif; font-weight: 700; letter-spacing: -0.36px;"
      >
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
            <div
              class="h-full bg-[#f64a46] rounded-full transition-all duration-300 progress-glow"
              style="width: {updateDownloadPercent}%"
            ></div>
          </div>
          {#if updateDownloaded > 0}
            <p class="text-[10px] text-white/20 mt-1">
              {(updateDownloaded / 1048576).toFixed(1)} МБ{updateTotal > 0
                ? ` / ${(updateTotal / 1048576).toFixed(1)} МБ`
                : ""}
            </p>
          {/if}
        </div>
      {:else}
        <!-- Кнопки -->
        <div class="flex gap-3">
          <button
            class="flex-1 px-4 py-2.5 rounded-lg bg-[#f64a46] hover:bg-[#ff5a56] active:scale-[0.97] text-sm text-white font-medium transition-all btn-ripple btn-bounce glow-hover"
            style="font-family: 'Proxima Nova Semibold', sans-serif; font-weight: 600;"
            onclick={handleInstallUpdate}
            onmouseenter={playHoverSound}
          >
            Скачать и установить
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

<style>
  :global(.progress-glow) {
    box-shadow: 0 0 10px rgba(246, 74, 70, 0.6);
  }
</style>
