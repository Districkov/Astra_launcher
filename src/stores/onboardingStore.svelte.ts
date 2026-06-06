/**
 * onboardingStore.svelte.ts — Онбординг (первый запуск)
 * Svelte 5 runes-based store
 */
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { handleError } from "../errorHandling";

// Lazy getters to avoid circular dependency at module load
function getServerStore() {
  return import("./serverStore.svelte").then(m => m.getServerStore());
}
function getFivemStore() {
  return import("./fivemStore.svelte").then(m => m.getFivemStore());
}

let onboardingActive = $state(false);
let onboardingStep = $state(1);
let onboardingFivemSearching = $state(false);
let onboardingFivemResult = $state<string | null>(null);
let onboardingNickname = $state("");
let onboardingNicknameSaving = $state(false);

async function checkOnboarding() {
  try {
    const complete = await invoke<boolean>("is_onboarding_complete");
    if (!complete) {
      onboardingActive = true;
      onboardingStep = 1;
    }
  } catch (e) {
    handleError(e, "checkOnboarding");
  }
}

async function onboardingNext(playClickSound: () => void, username: { value: string }, setUsername: (v: string) => void) {
  playClickSound();
  const fivem = await getFivemStore();

  if (onboardingStep === 1) {
    onboardingStep = 2;
    onboardingFivemSearching = true;
    onboardingFivemResult = null;
    try {
      const path = await invoke<string | null>("auto_find_fivem");
      if (path) {
        await fivem.loadFivemPath();
        onboardingFivemResult = 'found';
      } else {
        onboardingFivemResult = 'not_found';
      }
    } catch (e) {
      handleError(e, "onboardingNext.autoFind");
      onboardingFivemResult = 'not_found';
    }
    onboardingFivemSearching = false;
  } else if (onboardingStep === 2) {
    onboardingStep = 3;
  } else if (onboardingStep === 3) {
    onboardingNicknameSaving = true;
    const name = onboardingNickname.trim() || "Player";
    try {
      await invoke("save_username", { name });
      setUsername(name);
    } catch (e) {
      handleError(e, "onboardingNext.saveUsername");
    }
    try {
      await invoke("complete_onboarding");
    } catch (e) {
      handleError(e, "onboardingNext.completeOnboarding");
    }
    onboardingNicknameSaving = false;
    onboardingStep = 4;
  } else if (onboardingStep === 4) {
    onboardingActive = false;
  }
}

async function onboardingSelectFivem(playClickSound: () => void) {
  playClickSound();
  try {
    const selected = await open({
      multiple: false,
      filters: [{ name: "FiveM", extensions: ["exe"] }],
    });
    if (selected) {
      await invoke("set_fivem_path", { path: selected });
      const fivem = await getFivemStore();
      await fivem.loadFivemPath();
      onboardingFivemResult = 'found';
    }
  } catch (e) {
    handleError(e, "onboardingSelectFivem");
  }
}

async function onboardingDownloadFivem(playClickSound: () => void) {
  playClickSound();
  const fivem = await getFivemStore();
  await fivem.downloadAndInstall(playClickSound);
  onboardingFivemResult = fivem.found ? 'found' : null;
}

/** Go back one step, or close if on step 1 */
function onboardingBack() {
  if (onboardingStep > 1) {
    onboardingStep--;
  } else {
    onboardingActive = false;
  }
}

export function getOnboardingStore() {
  return {
    get active() { return onboardingActive; },
    get step() { return onboardingStep; },
    set step(v: number) { onboardingStep = v; },
    get fivemSearching() { return onboardingFivemSearching; },
    get fivemResult() { return onboardingFivemResult; },
    set fivemResult(v: string | null) { onboardingFivemResult = v; },
    get nickname() { return onboardingNickname; },
    set nickname(v: string) { onboardingNickname = v; },
    get nicknameSaving() { return onboardingNicknameSaving; },
    checkOnboarding,
    onboardingNext,
    onboardingSelectFivem,
    onboardingDownloadFivem,
    onboardingBack,
  };
}
