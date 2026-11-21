/**
 * onboardingStore.svelte.ts — Onboarding (first launch)
 * Svelte 5 runes-based store
 */
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { handleError } from "../errorHandling";

function getServerStore() {
  return import("./serverStore.svelte").then(m => m.getServerStore());
}
function getGameStore() {
  return import("./gameStore.svelte").then(m => m.getGameStore());
}

let onboardingActive = $state(false);
let onboardingStep = $state(1);
let onboardingRageSearching = $state(false);
let onboardingRageResult = $state<string | null>(null);
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
  const game = await getGameStore();

  if (onboardingStep === 1) {
    onboardingStep = 2;
    onboardingRageSearching = true;
    onboardingRageResult = null;
    try {
      const path = await invoke<string | null>("auto_find_rage_dir");
      if (path) {
        await game.loadRageDir();
        onboardingRageResult = 'found';
      } else {
        onboardingRageResult = 'not_found';
      }
    } catch (e) {
      handleError(e, "onboardingNext.autoFind");
      onboardingRageResult = 'not_found';
    }
    onboardingRageSearching = false;
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

async function onboardingSelectRageDir(playClickSound: () => void) {
  playClickSound();
  try {
    // Select the install directory, not an arbitrary .exe inside it.
    const selected = await open({
      directory: true,
      multiple: false,
      title: "Выберите папку RAGE:MP",
    });
    if (selected) {
      const result = await invoke<{ path: string; valid: boolean }>("set_rage_dir", {
        path: selected,
      });
      const game = await getGameStore();
      await game.loadRageDir();
      onboardingRageResult = result.valid ? "found" : "not_found";
    }
  } catch (e) {
    handleError(e, "onboardingSelectRageDir");
  }
}

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
    get rageSearching() { return onboardingRageSearching; },
    get rageResult() { return onboardingRageResult; },
    set rageResult(v: string | null) { onboardingRageResult = v; },
    get nickname() { return onboardingNickname; },
    set nickname(v: string) { onboardingNickname = v; },
    get nicknameSaving() { return onboardingNicknameSaving; },
    checkOnboarding,
    onboardingNext,
    onboardingSelectRageDir,
    onboardingBack,
  };
}
