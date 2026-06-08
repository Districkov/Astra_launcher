/**
 * storeStore.svelte — Стор магазина (донат, транспорт, имущество)
 * 
 * Пока API нет — товары захардкожены.
 * Когда API появится — заменить loadStoreItems() на fetch с сервера.
 */

import { invoke } from "@tauri-apps/api/core";
import type { StoreItem, StoreCategory, StoreCart } from "../types";
import { handleError } from "../errorHandling";

// ── Демо-товары (заменить на API) ──────────────

const DEMO_ITEMS: StoreItem[] = [
  // ── Донат ──
  {
    id: "vip-bronze",
    name: "VIP Бронза",
    description: "Доступ к VIP-командам, цвет ника в чате, эксклюзивный транспорт",
    price: 199,
    category: "donate",
    image: "/store/vip-bronze.png",
    badge: "Хит",
    popular: true,
  },
  {
    id: "vip-silver",
    name: "VIP Серебро",
    description: "Всё из Бронзы + персональный дом, /heal, увеличенный инвентарь",
    price: 399,
    oldPrice: 499,
    category: "donate",
    image: "/store/vip-silver.png",
    badge: "Скидка",
  },
  {
    id: "vip-gold",
    name: "VIP Золото",
    description: "Всё из Серебра + бизнес, /fly, уникальный скин, приоритет входа",
    price: 699,
    oldPrice: 899,
    category: "donate",
    image: "/store/vip-gold.png",
    badge: "Скидка",
    popular: true,
  },
  {
    id: "vip-platinum",
    name: "VIP Платина",
    description: "Максимальный уровень: все привилегии, уникальный транспорт, иммунитет",
    price: 1299,
    category: "donate",
    image: "/store/vip-platinum.png",
    badge: "Премиум",
  },
  {
    id: "money-500k",
    name: "500 000$",
    description: "Внутриигровая валюта — 500 000 долларов",
    price: 149,
    category: "donate",
    image: "/store/money.png",
  },
  {
    id: "money-1m",
    name: "1 000 000$",
    description: "Внутриигровая валюта — 1 000 000 долларов",
    price: 249,
    oldPrice: 299,
    category: "donate",
    image: "/store/money.png",
    badge: "Скидка",
  },
  {
    id: "money-5m",
    name: "5 000 000$",
    description: "Внутриигровая валюта — 5 000 000 долларов",
    price: 999,
    oldPrice: 1249,
    category: "donate",
    image: "/store/money.png",
    badge: "Скидка",
  },

  // ── Транспорт ──
  {
    id: "car-lamborghini",
    name: "Lamborghini Aventador",
    description: "Суперкар — максимальная скорость, уникальный дизайн",
    price: 599,
    category: "transport",
    image: "/store/car-lamborghini.png",
    badge: "Хит",
    popular: true,
  },
  {
    id: "car-bmw-m5",
    name: "BMW M5 F90",
    description: "Спортивный седан — комфорт и скорость",
    price: 349,
    category: "transport",
    image: "/store/car-bmw.png",
  },
  {
    id: "car-mercedes",
    name: "Mercedes-AMG GT",
    description: "Премиум спорткар — элегантность и мощь",
    price: 449,
    category: "transport",
    image: "/store/car-mercedes.png",
  },
  {
    id: "car-nissan-gtr",
    name: "Nissan GT-R R35",
    description: "Японский суперкар — легенда дрифта",
    price: 299,
    category: "transport",
    image: "/store/car-nissan.png",
  },
  {
    id: "bike-hayabusa",
    name: "Suzuki Hayabusa",
    description: "Мотоцикл — самая быстрая двухколёсная машина",
    price: 199,
    category: "transport",
    image: "/store/bike-hayabusa.png",
  },
  {
    id: "boat-yacht",
    name: "Люкс-яхта",
    description: "Роскошная яхта для морских прогулок",
    price: 799,
    category: "transport",
    image: "/store/boat-yacht.png",
    badge: "Премиум",
  },

  // ── Имущество ──
  {
    id: "house-small",
    name: "Уютный дом",
    description: "Небольшой дом в тихом районе, 2 комнаты, гараж",
    price: 299,
    category: "property",
    image: "/store/house-small.png",
  },
  {
    id: "house-medium",
    name: "Современная вилла",
    description: "Вилла с бассейном, 4 комнаты, большой гараж",
    price: 599,
    category: "property",
    image: "/store/house-medium.png",
    badge: "Хит",
    popular: true,
  },
  {
    id: "house-luxury",
    name: "Пентхаус",
    description: "Пентхаус на крыше небоскрёба, панорамный вид",
    price: 999,
    category: "property",
    image: "/store/house-luxury.png",
    badge: "Премиум",
  },
  {
    id: "business-247",
    name: "Магазин 24/7",
    description: "Бизнес — магазин 24/7, ежедневный доход",
    price: 449,
    category: "property",
    image: "/store/business-247.png",
  },
  {
    id: "business-ammunation",
    name: "Ammu-Nation",
    description: "Бизнес — оружейный магазин, высокий доход",
    price: 699,
    oldPrice: 799,
    category: "property",
    image: "/store/business-ammunation.png",
    badge: "Скидка",
  },
  {
    id: "business-carwash",
    name: "Автомойка",
    description: "Бизнес — автомойка, стабильный доход",
    price: 349,
    category: "property",
    image: "/store/business-carwash.png",
  },
];

// ── Состояние ──────────────────────────────────

let items = $state<StoreItem[]>([...DEMO_ITEMS]);
let activeCategory = $state<StoreCategory | "all">("all");
let searchQuery = $state("");
let loading = $state(false);
let cart = $state<StoreItem[]>([]);

// ── Геттеры ────────────────────────────────────

function getFilteredItems(): StoreItem[] {
  let filtered = items;

  if (activeCategory !== "all") {
    filtered = filtered.filter(item => item.category === activeCategory);
  }

  if (searchQuery.trim()) {
    const q = searchQuery.toLowerCase().trim();
    filtered = filtered.filter(
      item => item.name.toLowerCase().includes(q) || item.description.toLowerCase().includes(q)
    );
  }

  return filtered;
}

function getPopularItems(): StoreItem[] {
  return items.filter(item => item.popular);
}

function getCartTotal(): number {
  return cart.reduce((sum, item) => sum + item.price, 0);
}

function getCartCount(): number {
  return cart.length;
}

// ── Действия ───────────────────────────────────

function setActiveCategory(category: StoreCategory | "all") {
  activeCategory = category;
}

function setSearchQuery(query: string) {
  searchQuery = query;
}

function addToCart(item: StoreItem) {
  if (!cart.find(c => c.id === item.id)) {
    cart.push(item);
  }
}

function removeFromCart(itemId: string) {
  const idx = cart.findIndex(c => c.id === itemId);
  if (idx !== -1) {
    cart.splice(idx, 1);
  }
}

function clearCart() {
  cart.splice(0, cart.length);
}

async function purchaseItem(item: StoreItem) {
  try {
    // Открываем платёжный шлюз во встроенном webview
    if (item.paymentUrl) {
      await invoke("create_embedded_webview", { url: item.paymentUrl });
    } else {
      // Пока нет URL — открываем заглушку
      await invoke("create_embedded_webview", {
        url: `https://astra-rp.ru/store/pay?id=${item.id}`
      });
    }
  } catch (error: unknown) {
    handleError(error);
  }
}

async function purchaseCart() {
  if (cart.length === 0) return;

  try {
    // Формируем URL корзины с ID товаров
    const itemIds = cart.map(item => item.id).join(",");
    const total = getCartTotal();

    // Открываем платёжный шлюз со списком товаров
    await invoke("create_embedded_webview", {
      url: `https://astra-rp.ru/store/pay?items=${encodeURIComponent(itemIds)}&total=${total}`
    });

    // После открытия шлюза очищаем корзину
    clearCart();
  } catch (error: unknown) {
    handleError(error);
  }
}

// TODO: Заменить на API когда будет готов
async function loadStoreItems() {
  loading = true;
  try {
    // const response = await invoke<StoreItem[]>("get_store_items");
    // items = response;
    items = [...DEMO_ITEMS]; // Пока демо
  } catch (error: unknown) {
    handleError(error);
  } finally {
    loading = false;
  }
}

// ── Экспорт ────────────────────────────────────

export function getStoreStore() {
  return {
    // Геттеры
    get items() { return items; },
    get activeCategory() { return activeCategory; },
    get searchQuery() { return searchQuery; },
    get loading() { return loading; },
    get cart() { return cart; },
    get filteredItems() { return getFilteredItems(); },
    get popularItems() { return getPopularItems(); },
    get cartTotal() { return getCartTotal(); },
    get cartCount() { return getCartCount(); },

    // Действия
    setActiveCategory,
    setSearchQuery,
    addToCart,
    removeFromCart,
    clearCart,
    purchaseItem,
    purchaseCart,
    loadStoreItems,
  };
}
