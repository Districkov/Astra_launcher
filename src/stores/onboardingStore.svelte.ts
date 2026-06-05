/**
 * onboardingStore.svelte.ts — Онбординг (первый запуск)
 * Svelte 5 runes-based store
 */
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { listen } from "@tauri-apps/api/event";
import { getServerStore } from "./serverStore.svelte";
import { getFivemStore } from "./fivemStore.svelte";
import { fmtSize } from "../constants";

const { addNotification } = getServerStore();

let onboardingActive = $state(false);
let onboardingStep = $state(1);
let onboardingFivemSearching = $state(false);
let onboardingFivemResult = $state<string | null>(null);
let onboardingNickname = $state("");
let onboardingNicknameSaving = $state(false);

async function checkOnboarding() {
  try {
    const complete = await invoke("is_onboarding_complete") as boolean;
    if (!complete) {
      onboardingActive = true;
      onboardingStep = 1;
    }
  } catch (e) {
    // По умолчанию — не показываем
  }
}

async function onboardingNext(playClickSound: () => void, username: { value: string }, setUsername: (v: string) => void) {
  playClickSound();
  const fivem = getFivemStore();

  if (onboardingStep === 1) {
    onboardingStep = 2;
    onboardingFivemSearching = true;
    onboardingFivemResult = null;
    try {
      const path = await invoke("auto_find_fivem") as string | null;
      if (path) {
        // Update fivem store state through its methods
        await fivem.loadFivemPath();
        onboardingFivemResult = 'found';
      } else {
        onboardingFivemResult = 'not_found';
      }
    } catch (e) {
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
    } catch (e) {}
    try {
      await invoke("complete_onboarding");
    } catch (e) {}
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
      await getFivemStore().loadFivemPath();
      onboardingFivemResult = 'found';
    }
  } catch (e) {}
}

async function onboardingDownloadFivem(playClickSound: () => void) {
  playClickSound();
  const fivem = getFivemStore();
  // Use the shared download function from fivemStore
  await fivem.downloadAndInstall(playClickSound);
  // After download completes, check if installed
  onboardingFivemResult = fivem.found ? 'found' : null;
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
  };
}
