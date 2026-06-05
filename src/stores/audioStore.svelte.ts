/**
 * audioStore.svelte.ts — Звуки кликов и наведения
 * Svelte 5 runes-based store
 */

let clickSound: { play: () => void; currentTime: number } | null = null;
let hoverSound: { play: () => void; currentTime: number } | null = null;

function initAudio() {
  try {
    const audioCtx = new (window.AudioContext || (window as any).webkitAudioContext)();

    const generateClick = () => {
      const osc = audioCtx.createOscillator();
      const gain = audioCtx.createGain();
      osc.connect(gain);
      gain.connect(audioCtx.destination);
      osc.frequency.value = 800;
      osc.type = 'sine';
      gain.gain.setValueAtTime(0.15, audioCtx.currentTime);
      gain.gain.exponentialRampToValueAtTime(0.001, audioCtx.currentTime + 0.05);
      osc.start(audioCtx.currentTime);
      osc.stop(audioCtx.currentTime + 0.05);
    };
    clickSound = { play: generateClick, currentTime: 0 };

    const generateHover = () => {
      const osc = audioCtx.createOscillator();
      const gain = audioCtx.createGain();
      osc.connect(gain);
      gain.connect(audioCtx.destination);
      osc.frequency.value = 600;
      osc.type = 'sine';
      gain.gain.setValueAtTime(0.25, audioCtx.currentTime);
      gain.gain.exponentialRampToValueAtTime(0.001, audioCtx.currentTime + 0.04);
      osc.start(audioCtx.currentTime);
      osc.stop(audioCtx.currentTime + 0.04);
    };
    hoverSound = { play: generateHover, currentTime: 0 };
  } catch (e) {
    clickSound = null;
    hoverSound = null;
  }
}

function playClickSound() {
  if (!clickSound) return;
  try { clickSound.play(); } catch (e) {}
}

function playHoverSound() {
  if (!hoverSound) return;
  try { hoverSound.play(); } catch (e) {}
}

export function getAudioStore() {
  return {
    initAudio,
    playClickSound,
    playHoverSound,
  };
}
