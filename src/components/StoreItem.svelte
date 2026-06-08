<!--
  StoreItem.svelte — Карточка товара магазина
-->
<script lang="ts">
  import type { StoreItem } from "../types";

  let {
    item = $bindable(),
    onBuy = () => {},
    onAddToCart = () => {},
    playHoverSound = () => {},
    inCart = false,
  }: {
    item: StoreItem;
    onBuy?: () => void;
    onAddToCart?: () => void;
    playHoverSound?: () => void;
    inCart?: boolean;
  } = $props();

  let hovered = $state(false);

  const categoryLabels: Record<string, string> = {
    donate: "Донат",
    transport: "Транспорт",
    property: "Имущество",
  };

  const badgeColors: Record<string, string> = {
    "Хит": "bg-yellow-500/20 text-yellow-400 border-yellow-500/30",
    "Скидка": "bg-green-500/20 text-green-400 border-green-500/30",
    "Новинка": "bg-blue-500/20 text-blue-400 border-blue-500/30",
    "Премиум": "bg-purple-500/20 text-purple-400 border-purple-500/30",
  };
</script>

<!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
<div
  class="relative group bg-white/[0.03] border border-white/[0.06] rounded-xl overflow-hidden transition-all duration-200 cursor-pointer
    {hovered ? 'border-[#f64a46]/30 bg-white/[0.05] shadow-lg shadow-[#f64a46]/5' : ''}"
  onmouseenter={() => { hovered = true; playHoverSound(); }}
  onmouseleave={() => { hovered = false; }}
  onclick={onBuy}
>
  <!-- Бейдж -->
  {#if item.badge}
    <div class="absolute top-2.5 right-2.5 z-10">
      <span class="text-[10px] font-semibold px-2 py-0.5 rounded-full border {badgeColors[item.badge] || 'bg-white/10 text-white/50 border-white/10'}">
        {item.badge}
      </span>
    </div>
  {/if}

  <!-- Картинка -->
  <div class="relative h-[120px] bg-gradient-to-br from-white/[0.02] to-white/[0.05] flex items-center justify-center overflow-hidden">
    {#if item.image}
      <img src={item.image} alt={item.name} class="w-full h-full object-cover transition-transform duration-300 {hovered ? 'scale-110' : ''}" />
    {:else}
      <!-- Заглушка если нет картинки -->
      <div class="text-white/10">
        {#if item.category === 'donate'}
          <svg class="w-12 h-12" viewBox="0 0 24 24" fill="currentColor"><path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm1 17h-2v-2h2v2zm0-4h-2V7h2v8z"/></svg>
        {:else if item.category === 'transport'}
          <svg class="w-12 h-12" viewBox="0 0 24 24" fill="currentColor"><path d="M18.92 6.01C18.72 5.42 18.16 5 17.5 5h-11c-.66 0-1.21.42-1.42 1.01L3 12v8c0 .55.45 1 1 1h1c.55 0 1-.45 1-1v-1h12v1c0 .55.45 1 1 1h1c.55 0 1-.45 1-1v-8l-2.08-5.99zM6.5 16c-.83 0-1.5-.67-1.5-1.5S5.67 13 6.5 13s1.5.67 1.5 1.5S7.33 16 6.5 16zm11 0c-.83 0-1.5-.67-1.5-1.5s.67-1.5 1.5-1.5 1.5.67 1.5 1.5-.67 1.5-1.5 1.5zM5 11l1.5-4.5h11L19 11H5z"/></svg>
        {:else}
          <svg class="w-12 h-12" viewBox="0 0 24 24" fill="currentColor"><path d="M10 20v-4h4v4h-4zm-6 0v-4h4v4H4zm12 0v-4h4v4h-4zm-6-6v-4h4v4h-4zm-6 0v-4h4v4H4zm12 0v-4h4v4h-4zM10 8V4h4v4h-4zM4 8V4h4v4H4zm12 0V4h4v4h-4z"/></svg>
        {/if}
      </div>
    {/if}

    <!-- Оверлей при наведении -->
    <div class="absolute inset-0 bg-gradient-to-t from-black/60 via-transparent to-transparent opacity-0 group-hover:opacity-100 transition-opacity duration-200"></div>
  </div>

  <!-- Информация -->
  <div class="p-3">
    <!-- Категория -->
    <span class="text-[10px] text-white/20 uppercase tracking-wider">{categoryLabels[item.category] || item.category}</span>

    <!-- Название -->
    <h3 class="text-sm font-semibold text-white/90 mt-0.5 leading-tight" style="font-family: 'Proxima Nova Bold', sans-serif;">
      {item.name}
    </h3>

    <!-- Описание -->
    <p class="text-[11px] text-white/30 mt-1 leading-snug line-clamp-2">{item.description}</p>

    <!-- Цена и кнопка -->
    <div class="flex items-center justify-between mt-3">
      <div class="flex items-baseline gap-1.5">
        <span class="text-base font-bold text-white" style="font-family: 'Proxima Nova Bold', sans-serif;">
          {item.price}₽
        </span>
        {#if item.oldPrice}
          <span class="text-[11px] text-white/20 line-through">{item.oldPrice}₽</span>
        {/if}
      </div>

      <button
        class="text-[11px] font-semibold px-3 py-1.5 rounded-lg transition-all duration-150
          {inCart
            ? 'bg-green-500/20 text-green-400 border border-green-500/30'
            : 'bg-[#f64a46]/20 text-[#ff8c8c] border border-[#f64a46]/30 hover:bg-[#f64a46]/30'}"
        onclick={(e) => { e.stopPropagation(); onAddToCart(); }}
        onmouseenter={playHoverSound}
      >
        {inCart ? '✓ В корзине' : 'В корзину'}
      </button>
    </div>
  </div>
</div>
