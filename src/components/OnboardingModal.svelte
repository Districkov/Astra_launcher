<!--
  OnboardingModal.svelte — Модалка онбординга (4 шага: приветствие, FiveM, никнейм, готово)
-->
<script>
  let {
    onboardingActive = false,
    onboardingStep = 1,
    onboardingFivemSearching = false,
    onboardingFivemResult = null,
    onboardingNickname = "",
    onboardingNicknameSaving = false,
    fivemPath = "",
    isDownloading = false,
    downloadPercent = 0,
    downloadSize = "",
    downloadError = "",
    statusMessage = "",
    onboardingNext = () => {},
    onboardingSelectFivem = async () => {},
    onboardingDownloadFivem = async () => {},
    playHoverSound = () => {},
    onNicknameChange = () => {},
  } = $props();
</script>

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
              <div class="w-full max-w-xs mb-4">
                <p class="text-xs text-white/40 mb-2 text-center">{downloadSize || "Скачивание FiveM…"}</p>
                <div class="w-full h-2 bg-white/10 rounded-full overflow-hidden">
                  <div class="h-full bg-gradient-to-r from-[#f64a46] to-[#ff8c4d] rounded-full transition-all duration-300"
                       style="width: {downloadPercent}%"></div>
                </div>
                <p class="text-xs text-white/20 mt-1 text-center">{downloadPercent}%</p>
              </div>
              {#if statusMessage}
                <p class="text-xs text-white/50 mb-3 text-center max-w-xs">{statusMessage}</p>
              {/if}
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
              value={onboardingNickname}
              oninput={(e) => { onNicknameChange(e.target.value); }}
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
