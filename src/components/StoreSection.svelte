<!--
  StoreSection.svelte — Секция «Магазин» (категории, товары, корзина, платёжный шлюз)
-->
<script lang="ts">
  import type { StoreItem, StoreCategory } from "../types";
  import StoreItemComponent from "./StoreItem.svelte";

  let {
    store = $bindable(),
    playHoverSound = () => {},
    playClickSound = () => {},
    openEmbeddedBrowser = async () => {},
  }: {
    store: ReturnType<typeof import("../stores/storeStore.svelte").getStoreStore>;
    playHoverSound?: () => void;
    playClickSound?: () => void;
    openEmbeddedBrowser?: (url: string) => Promise<void>;
  } = $props();

  const categories: { id: StoreCategory | "all"; label: string; icon: string }[] = [
    { id: "all", label: "Все", icon: "all" },
    { id: "donate", label: "Донат", icon: "donate" },
    { id: "transport", label: "Транспорт", icon: "transport" },
    { id: "property", label: "Имущество", icon: "property" },
  ];

  let showCart = $state(false);

  function handleBuy(item: StoreItem) {
    playClickSound();
    store.purchaseItem(item);
  }

  function handleAddToCart(item: StoreItem) {
    playClickSound();
    store.addToCart(item);
  }

  function handleRemoveFromCart(itemId: string) {
    playClickSound();
    store.removeFromCart(itemId);
  }

  function handlePurchaseCart() {
    playClickSound();
    store.purchaseCart();
  }

  function isInCart(itemId: string): boolean {
    return store.cart.some(c => c.id === itemId);
  }
</script>

<div class="w-full h-full flex flex-col overflow-hidden">
  <!-- Шапка магазина -->
  <div class="flex items-center justify-between px-6 pt-5 pb-3">
    <div>
      <h2 class="text-xl font-bold text-white" style="font-family: 'Proxima Nova Bold', sans-serif;">
        Магазин
      </h2>
      <p class="text-xs text-white/20 mt-0.5">Привилегии, транспорт и имущество для ASTRA RP</p>
    </div>

    <!-- Корзина -->
    <button
      class="relative flex items-center gap-2 px-3 py-2 rounded-lg bg-white/[0.03] border border-white/[0.06] text-white/50 hover:text-white/80 hover:border-[#f64a46]/20 transition-all duration-150"
      onclick={() => { playClickSound(); showCart = !showCart; }}
      onmouseenter={playHoverSound}
    >
      <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <circle cx="9" cy="21" r="1"/><circle cx="20" cy="21" r="1"/>
        <path d="M1 1h4l2.68 13.39a2 2 0 0 0 2 1.61h9.72a2 2 0 0 0 2-1.61L23 6H6"/>
      </svg>
      <span class="text-sm" style="font-family: 'Proxima Nova Semibold', sans-serif;">
        {store.cartCount > 0 ? `${store.cartCount} · ${store.cartTotal}₽` : 'Корзина'}
      </span>
      {#if store.cartCount > 0}
        <span class="absolute -top-1.5 -right-1.5 w-4 h-4 bg-[#f64a46] rounded-full text-[9px] font-bold text-white flex items-center justify-center">
          {store.cartCount}
        </span>
      {/if}
    </button>
  </div>

  <!-- Категории + Поиск -->
  <div class="flex items-center gap-2 px-6 pb-3">
    {#each categories as cat}
      <button
        class="px-3 py-1.5 rounded-lg text-xs font-semibold transition-all duration-150
          {store.activeCategory === cat.id
            ? 'bg-[#f64a46]/20 text-[#ff8c8c] border border-[#f64a46]/30'
            : 'bg-white/[0.03] text-white/30 border border-white/[0.06] hover:text-white/60 hover:border-white/10'}"
        style="font-family: 'Proxima Nova Semibold', sans-serif;"
        onclick={() => { playClickSound(); store.setActiveCategory(cat.id); }}
        onmouseenter={playHoverSound}
      >
        {cat.label}
      </button>
    {/each}

    <!-- Поиск -->
    <div class="ml-auto relative">
      <svg class="absolute left-2.5 top-1/2 -translate-y-1/2 w-3.5 h-3.5 text-white/20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <circle cx="11" cy="11" r="8"/><path d="m21 21-4.35-4.35"/>
      </svg>
      <input
        type="text"
        placeholder="Поиск..."
        value={store.searchQuery}
        oninput={(e) => store.setSearchQuery(e.target.value)}
        class="w-[160px] pl-8 pr-3 py-1.5 rounded-lg bg-white/[0.03] border border-white/[0.06] text-xs text-white/60 placeholder-white/20 focus:outline-none focus:border-[#f64a46]/20 transition-colors"
        style="font-family: 'Proxima Nova', sans-serif;"
      />
    </div>
  </div>

  <!-- Товары / Корзина -->
  {#if showCart}
    <!-- ═══════════════════════════════════════
         КОРЗИНА
         ═══════════════════════════════════════ -->
    <div class="flex-1 overflow-y-auto px-6 pb-4 scrollbar-thin">
      {#if store.cart.length === 0}
        <div class="flex flex-col items-center justify-center h-full text-white/20">
          <svg class="w-16 h-16 mb-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1" stroke-linecap="round" stroke-linejoin="round">
            <circle cx="9" cy="21" r="1"/><circle cx="20" cy="21" r="1"/>
            <path d="M1 1h4l2.68 13.39a2 2 0 0 0 2 1.61h9.72a2 2 0 0 0 2-1.61L23 6H6"/>
          </svg>
          <p class="text-sm">Корзина пуста</p>
          <p class="text-xs mt-1">Добавьте товары из каталога</p>
        </div>
      {:else}
        <div class="space-y-2">
          {#each store.cart as cartItem (cartItem.id)}
            <div class="flex items-center gap-3 p-3 rounded-lg bg-white/[0.03] border border-white/[0.06]">
              <div class="flex-1 min-w-0">
                <h4 class="text-sm font-semibold text-white/80" style="font-family: 'Proxima Nova Bold', sans-serif;">{cartItem.name}</h4>
                <p class="text-[11px] text-white/30 mt-0.5 truncate">{cartItem.description}</p>
              </div>
              <span class="text-sm font-bold text-white" style="font-family: 'Proxima Nova Bold', sans-serif;">{cartItem.price}₽</span>
              <button
                class="w-7 h-7 rounded-lg bg-red-500/10 border border-red-500/20 text-red-400 hover:bg-red-500/20 transition-colors flex items-center justify-center"
                onclick={() => handleRemoveFromCart(cartItem.id)}
                onmouseenter={playHoverSound}
              >
                <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M18 6L6 18M6 6l12 12"/></svg>
              </button>
            </div>
          {/each}
        </div>

        <!-- Итого -->
        <div class="mt-4 p-4 rounded-xl bg-[#f64a46]/5 border border-[#f64a46]/10">
          <div class="flex items-center justify-between">
            <span class="text-sm text-white/50">Итого:</span>
            <span class="text-lg font-bold text-white" style="font-family: 'Proxima Nova Bold', sans-serif;">{store.cartTotal}₽</span>
          </div>
          <button
            class="w-full mt-3 py-2.5 rounded-lg bg-[#f64a46] hover:bg-[#f64a46]/80 text-white text-sm font-semibold transition-colors"
            style="font-family: 'Proxima Nova Bold', sans-serif;"
            onclick={handlePurchaseCart}
            onmouseenter={playHoverSound}
          >
            Оплатить
          </button>
        </div>
      {/if}
    </div>
  {:else}
    <!-- ═══════════════════════════════════════
         КАТАЛОГ ТОВАРОВ
         ═══════════════════════════════════════ -->
    <div class="flex-1 overflow-y-auto px-6 pb-4 scrollbar-thin">
      {#if store.loading}
        <!-- Скелетон загрузки -->
        <div class="grid grid-cols-3 gap-3">
          {#each Array(6) as _, i}
            <div class="rounded-xl bg-white/[0.03] border border-white/[0.06] overflow-hidden animate-skeleton">
              <div class="h-[120px] bg-white/[0.02]"></div>
              <div class="p-3 space-y-2">
                <div class="h-3 w-12 rounded bg-white/[0.05]"></div>
                <div class="h-4 w-24 rounded bg-white/[0.05]"></div>
                <div class="h-3 w-full rounded bg-white/[0.03]"></div>
                <div class="h-3 w-2/3 rounded bg-white/[0.03]"></div>
              </div>
            </div>
          {/each}
        </div>
      {:else if store.filteredItems.length === 0}
        <div class="flex flex-col items-center justify-center h-full text-white/20">
          <svg class="w-12 h-12 mb-2" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1"><circle cx="11" cy="11" r="8"/><path d="m21 21-4.35-4.35"/></svg>
          <p class="text-sm">Ничего не найдено</p>
        </div>
      {:else}
        <!-- Популярные (если категория "all" и нет поиска) -->
        {#if store.activeCategory === "all" && !store.searchQuery && store.popularItems.length > 0}
          <div class="mb-4">
            <h3 class="text-xs text-white/20 uppercase tracking-wider mb-2" style="font-family: 'Proxima Nova Semibold', sans-serif;">Популярное</h3>
            <div class="grid grid-cols-3 gap-3">
              {#each store.popularItems as item (item.id)}
                <StoreItemComponent
                  {item}
                  onBuy={() => handleBuy(item)}
                  onAddToCart={() => handleAddToCart(item)}
                  {playHoverSound}
                  inCart={isInCart(item.id)}
                />
              {/each}
            </div>
          </div>
        {/if}

        <!-- Все товары -->
        <div>
          {#if store.activeCategory === "all" && !store.searchQuery}
            <h3 class="text-xs text-white/20 uppercase tracking-wider mb-2" style="font-family: 'Proxima Nova Semibold', sans-serif;">Все товары</h3>
          {/if}
          <div class="grid grid-cols-3 gap-3">
            {#each store.filteredItems as item (item.id)}
              <StoreItemComponent
                {item}
                onBuy={() => handleBuy(item)}
                onAddToCart={() => handleAddToCart(item)}
                {playHoverSound}
                inCart={isInCart(item.id)}
              />
            {/each}
          </div>
        </div>
      {/if}
    </div>
  {/if}
</div>
