<script lang="ts">
  import type { App } from "../types";

  export let activeMenu: string;
  export let serverOnline: boolean;
  export let serverLoading: boolean;
  export let serverPlayers: number;
  export let serverPing: number;
  export let username: string;
  export let appWindow: any;
  export let menuItems: Array<{ id: string; label: string; icon: string; group?: string }>;

  export let selectMenu: (id: string) => void;
  export let playHoverSound: () => void;
  export let playClickSound: () => void;
  export let addNotification: (message: string, type: string) => void;
</script>

<aside class="absolute top-0 left-0 w-[157px] h-full bg-[#1b1b1b] rounded-l-[5px] z-20 flex flex-col">
  <!-- Логотип ASTRA (drag-зона) -->
  <div
    data-tauri-drag-region
    role="none"
    class="pt-[26px] pl-[44px] cursor-default"
    onmousedown={() => {
      if (appWindow) appWindow.startDragging();
    }}
  >
    <div
      class="text-white tracking-[-0.8px] leading-none"
      style="font-family: 'Armor Piercing 2.0 BB', 'Impact', sans-serif; font-size: clamp(32px, 4.2vw, 40px);"
    >
      ASTRA
    </div>
    <!-- Красная полоска под логотипом -->
    <div class="-mt-[3px] w-[65px] h-[3px] bg-[#f64a46] rounded-[21px]"></div>
  </div>

  <!-- Статус онлайна -->
  <div class="mt-[40px] pl-[31px] flex items-center gap-2">
    {#if serverLoading}
      <div class="w-3 h-3 rounded-md bg-yellow-500/20 flex items-center justify-center">
        <div class="w-2 h-2 rounded bg-yellow-500 animate-pulse"></div>
      </div>
      <div class="flex flex-col gap-1.5">
        <div class="w-14 h-2.5 rounded bg-white/10 animate-skeleton"></div>
        <div class="w-8 h-2 rounded bg-white/5 animate-skeleton"></div>
      </div>
    {:else if serverOnline}
      <div class="w-3 h-3 rounded-md bg-green-500/20 flex items-center justify-center">
        <div class="w-2 h-2 rounded bg-[#15ff00]"></div>
      </div>
      <span
        class="text-sm"
        style="font-family: 'Proxima Nova Bold', sans-serif; font-weight: 700; letter-spacing: -0.04px;"
      >
        <span class="text-white/30">Онлайн:</span>
        <span class="text-white"> {serverPlayers}</span>
        {#if serverPing > 0}
          <span class="text-white/20 ml-1">|</span>
          <span class="text-white/30 ml-1" style="font-size: 11px;">
            {#if serverPing < 50}
              <span class="text-green-400">{serverPing}ms</span>
            {:else if serverPing < 100}
              <span class="text-yellow-400">{serverPing}ms</span>
            {:else}
              <span class="text-red-400">{serverPing}ms</span>
            {/if}
          </span>
        {/if}
      </span>
    {:else}
      <div class="w-3 h-3 rounded-md bg-red-500/20 flex items-center justify-center">
        <div class="w-2 h-2 rounded bg-red-500"></div>
      </div>
      <span class="text-sm text-red-400" style="font-family: 'Proxima Nova Bold', sans-serif; font-weight: 700;">
        Офлайн
      </span>
    {/if}
  </div>

  <!-- Навигация -->
  <nav class="mt-[35px] flex-1">
    {#each menuItems as item, i}
      {#if i > 0 && item.group !== menuItems[i - 1].group}
        <div class="my-3"></div>
      {/if}
      <button
        class="menu-item w-full flex items-center gap-3 pl-[34px] h-[42px] text-[15px] transition-colors duration-150 relative
          {activeMenu === item.id ? 'text-white menu-item-active' : 'text-white/30 hover:text-white/60 menu-item-hover'}"
        style="font-family: 'Proxima Nova Semibold', sans-serif; font-weight: 600; letter-spacing: -0.30px;
          {activeMenu === item.id ? 'background: linear-gradient(90deg, rgba(246,74,70,0.5) 0%, rgba(0,0,0,0.1) 100%)' : ''}"
        onclick={() => selectMenu(item.id)}
        onmouseenter={playHoverSound}
      >
        {#if item.icon === "play"}
          <svg class="w-3 h-3" viewBox="0 0 16 17" fill="currentColor"
            ><polygon points="0,0 16,8.5 0,17" /></svg
          >
        {:else if item.icon === "store"}
          <svg class="w-4 h-4" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M0.75 14.75H14.75M10.08 4.64C10.08 5.26 10.33 5.85 10.77 6.29C11.2 6.73 11.8 6.97 12.42 6.97C13.04 6.97 13.63 6.73 14.07 6.29C14.5 5.85 14.75 5.26 14.75 4.64V3.86L13.19 0.75H2.31L0.75 3.86V4.64C0.75 5.26 1 5.85 1.43 6.29C1.87 6.73 2.46 6.97 3.08 6.97C3.7 6.97 4.3 6.73 4.73 6.29C5.17 5.85 5.42 5.26 5.42 4.64M5.42 3.86V4.64C5.42 5.26 5.66 5.85 6.1 6.29C6.54 6.73 7.13 6.97 7.75 6.97C8.37 6.97 8.96 6.73 9.4 6.29C9.84 5.85 10.08 5.26 10.08 4.64V3.86M2.31 14.75V6.86M13.19 14.75V6.86M5.42 14.75V11.64C5.42 11.23 5.58 10.83 5.87 10.54C6.16 10.25 6.56 10.08 6.97 10.08H8.53C8.94 10.08 9.34 10.25 9.63 10.54C9.92 10.83 10.08 11.23 10.08 11.64V14.75" /></svg>
        {:else if item.icon === "news"}
          <svg class="w-4 h-4" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M11.25 2.5H13.88C14.11 2.5 14.33 2.59 14.49 2.76C14.66 2.92 14.75 3.14 14.75 3.38V13C14.75 13.46 14.57 13.91 14.24 14.24C13.91 14.57 13.46 14.75 13 14.75M11.25 2.5V13C11.25 13.46 11.43 13.91 11.76 14.24C12.09 14.57 12.54 14.75 13 14.75M11.25 2.5V1.63C11.25 1.39 11.16 1.17 10.99 1.01C10.83 0.84 10.61 0.75 10.38 0.75H1.63C1.39 0.75 1.17 0.84 1.01 1.01C0.84 1.17 0.75 1.39 0.75 1.63V12.13C0.75 12.82 1.03 13.49 1.52 13.98C2.01 14.47 2.68 14.75 3.38 14.75H13M4.25 4.25H7.75M4.25 7.75H7.75M4.25 11.25H7.75" /></svg>
        {:else if item.icon === "forum"}
          <svg class="w-4 h-4" viewBox="0 0 16 15" fill="none" stroke="currentColor" stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round"><path d="M4.74 4.67H10.96M4.74 7.73H9.41M5.52 11.56H3.18C2.56 11.56 1.97 11.31 1.53 10.88C1.1 10.45 0.85 9.87 0.85 9.26V3.14C0.85 2.54 1.1 1.95 1.53 1.52C1.97 1.09 2.56 0.85 3.18 0.85H12.52C13.14 0.85 13.73 1.09 14.17 1.52C14.6 1.95 14.85 2.54 14.85 3.14V9.26C14.85 9.87 14.6 10.45 14.17 10.88C14.73 11.31 13.14 11.56 12.52 11.56H10.18L7.85 13.85L5.52 11.56Z" /></svg>
        {:else if item.icon === "discord"}
          <svg class="w-3.5 h-3" viewBox="0 0 14 11" fill="currentColor"><path d="M11.86 0.92C10.95 0.49 9.98 0.18 8.97 0C8.85 0.23 8.7 0.54 8.6 0.78C7.52 0.62 6.46 0.62 5.4 0.78C5.3 0.54 5.15 0.23 5.03 0C4.01 0.18 3.04 0.49 2.14 0.92C0.31 3.73 -0.19 6.46 0.06 9.16C1.27 10.08 2.45 10.63 3.6 11C3.89 10.6 4.14 10.18 4.36 9.73C3.94 9.57 3.54 9.37 3.17 9.14C3.27 9.07 3.36 8.99 3.46 8.91C5.76 10 8.27 10 10.54 8.91C10.64 8.99 10.74 9.07 10.83 9.14C10.46 9.37 10.05 9.57 9.64 9.73C9.86 10.18 10.11 10.6 10.4 11C11.55 10.63 12.73 10.08 13.94 9.16C14.23 6.03 13.44 3.32 11.86 0.92ZM4.67 7.5C3.98 7.5 3.42 6.84 3.42 6.05C3.42 5.25 3.97 4.59 4.67 4.59C5.38 4.59 5.95 5.25 5.93 6.05C5.93 6.84 5.38 7.5 4.67 7.5ZM9.33 7.5C8.63 7.5 8.07 6.84 8.07 6.05C8.07 5.25 8.62 4.59 9.33 4.59C10.03 4.59 10.6 5.25 10.58 6.05C10.58 6.84 10.03 7.5 9.33 7.5Z" /></svg>
        {:else if item.icon === "settings"}
          <svg class="w-3.5 h-3" viewBox="0 0 14 13" fill="currentColor"><path d="M12.66 4.59C11.4 4.59 10.88 3.7 11.51 2.62C11.87 2 11.66 1.2 11.02 0.84L9.81 0.16C9.26 -0.17 8.54 0.03 8.21 0.57L8.14 0.7C7.51 1.78 6.47 1.78 5.84 0.7L5.76 0.57C5.44 0.03 4.73 -0.17 4.18 0.16L2.97 0.84C2.33 1.2 2.11 2 2.48 2.63C3.11 3.7 2.6 4.59 1.33 4.59C0.6 4.59 0 5.17 0 5.89V7.11C0 7.82 0.59 8.41 1.33 8.41C2.6 8.41 3.11 9.3 2.48 10.38C2.11 11 2.33 11.8 2.97 12.16L4.18 12.84C4.73 13.17 5.44 12.97 5.77 12.43L5.85 12.3C6.48 11.22 7.51 11.22 8.15 12.3L8.23 12.43C8.56 12.97 9.27 13.17 9.82 12.84L11.03 12.16C11.67 11.8 11.89 11 11.52 10.38C10.89 9.3 11.4 8.41 12.67 8.41C13.4 8.41 14 7.83 14 7.11V5.89C14 5.18 13.4 4.59 12.66 4.59ZM7 8.74C5.74 8.74 4.72 7.73 4.72 6.5C4.72 5.27 5.74 4.26 7 4.26C8.25 4.26 9.27 5.27 9.27 6.5C9.27 7.73 8.25 8.74 7 8.74Z" /></svg>
        {/if}
        <span class="whitespace-nowrap">{item.label}</span>
      </button>
    {/each}
  </nav>

  <!-- Разделитель -->
  <div class="mx-4 mb-2 h-px bg-white/10"></div>

  <!-- Профиль пользователя -->
  <div class="px-4 pb-3 flex flex-col items-center">
    <div class="flex items-center gap-2 mb-2">
      <svg class="w-6 h-6 text-white/30" viewBox="0 0 22 25" fill="currentColor"
        ><path d="M20.09 4.81L12.89 0.53C11.72 -0.18 10.27 -0.18 9.08 0.53L1.9 4.81C0.73 5.51 0 6.81 0 8.23V16.78C0 18.18 0.73 19.48 1.9 20.19L9.09 24.48C10.27 25.18 11.72 25.18 12.91 24.48L20.1 20.19C21.27 19.49 22 18.19 22 16.78V8.23C21.99 6.81 21.26 5.53 20.09 4.81ZM10.99 6.68C12.56 6.68 13.82 7.98 13.82 9.59C13.82 11.2 12.56 12.5 10.99 12.5C9.43 12.5 8.17 11.2 8.17 9.59C8.17 7.99 9.43 6.68 10.99 6.68ZM14.24 18.33H7.75C6.77 18.33 6.2 17.2 6.74 16.36C7.57 15.1 9.17 14.25 10.99 14.25C12.82 14.25 14.42 15.1 15.24 16.36C15.79 17.19 15.21 18.33 14.24 18.33Z" /></svg>
      <p class="text-sm text-white" style="font-family: 'Proxima Nova Bold', sans-serif; font-weight: 700; letter-spacing: -0.28px;">
        {username}
      </p>
    </div>
    <button
      class="px-4 py-1 text-xs text-white/30 hover:text-white/60 hover:bg-white/5 rounded transition-colors btn-ripple"
      onclick={(e) => {
        e.stopPropagation();
        playClickSound();
        addNotification("Вы вышли из аккаунта", "info");
      }}
      onmouseenter={playHoverSound}
    >
      Выйти
    </button>
  </div>
</aside>

<style>
  :global(.menu-item) {
    position: relative;
    overflow: hidden;
  }

  :global(.menu-item-active) {
    font-weight: 700 !important;
  }

  :global(.menu-item::before) {
    content: "";
    position: absolute;
    left: 0;
    width: 3px;
    height: 0;
    background: #f64a46;
    transition: height 0.3s ease;
  }

  :global(.menu-item-active::before) {
    height: 100%;
  }
</style>
