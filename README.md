# Astra Launcher

![Tauri](https://img.shields.io/badge/Tauri-2-24C8DB?style=flat-square&logo=tauri&logoColor=white)
![Svelte](https://img.shields.io/badge/Svelte-5-FF3E00?style=flat-square&logo=svelte&logoColor=white)
![TypeScript](https://img.shields.io/badge/TypeScript-5.4-3178C6?style=flat-square&logo=typescript&logoColor=white)
![Rust](https://img.shields.io/badge/Rust-000000?style=flat-square&logo=rust&logoColor=white)
![Tailwind CSS](https://img.shields.io/badge/Tailwind_CSS-3-06B6D4?style=flat-square&logo=tailwind-css&logoColor=white)

Десктопный лаунчер для игрового проекта: интерфейс на **Svelte 5**, нативная оболочка
на **Tauri 2 (Rust)**. Запуск игры, магазин, настройки, онбординг и
**автообновление с проверкой подписи**.

**Версия:** 1.9.10 · **Платформа:** Windows (x64)

## ✨ Возможности

- 🎮 **Запуск игры** одной кнопкой — раздел «Играть»
- 🛍 **Магазин** с карточками товаров и разделами
- ⚙️ **Настройки** с сохранением пользовательских параметров
- 🔄 **Автообновление** — проверка версии, модальное окно обновления и индикатор статуса; релизы подписываются (`latest.json.sig`), что защищает от подмены файлов
- 🚪 **Онбординг** для новых пользователей
- 🌍 **Локализация (i18n)** — интерфейс поддерживает несколько языков
- 🔔 **Уведомления** и модальные окна подтверждения (закрытие, выход)
- 🎧 **Управление звуком** — отдельное хранилище состояния аудио
- 🪟 **Кастомные элементы окна** — собственная панель управления и фон
- 💻 **Нативное окно** вместо браузерной вкладки — Tauri вместо Electron, поэтому размер и потребление памяти заметно ниже

## 🛠 Стек

| Слой | Технологии |
|------|-----------|
| Интерфейс | Svelte 5, TypeScript 5.4, Tailwind CSS 3 |
| Состояние | Svelte stores (`*.svelte.ts`) |
| Сборка | Vite 5, `@sveltejs/vite-plugin-svelte` |
| Десктоп-оболочка | **Tauri 2**, Rust |
| Плагины Tauri | `updater` (автообновление), `dialog`, `process`, `shell` |
| Качество | ESLint 8 (`eslint-plugin-svelte`), Prettier (`prettier-plugin-svelte`) |

## 🏗 Архитектура

```
Svelte-интерфейс (src/)
        │  Svelte stores — единое состояние приложения
        ▼
Tauri API (@tauri-apps/api)
        │  IPC-мост
        ▼
Rust-ядро (src-tauri/)
        │
        ├── tauri.conf.json   — конфиг приложения и окна
        ├── capabilities/     — разрешения (permissions) для плагинов
        └── icons/            — иконки приложения
```

## 📁 Структура проекта

```
Astra_launcher/
├── src/                          # интерфейс на Svelte
│   ├── components/
│   │   ├── Sidebar.svelte            # боковая навигация
│   │   ├── PlaySection.svelte        # запуск игры
│   │   ├── StoreSection.svelte       # магазин
│   │   ├── StoreItem.svelte          # карточка товара
│   │   ├── Settings.svelte           # настройки
│   │   ├── OnboardingModal.svelte    # онбординг
│   │   ├── UpdateModal.svelte        # окно обновления
│   │   ├── UpdateIndicator.svelte    # индикатор версии
│   │   ├── Notifications.svelte      # уведомления
│   │   ├── CloseConfirmModal.svelte  # подтверждение закрытия
│   │   ├── LogoutConfirmModal.svelte # подтверждение выхода
│   │   ├── WindowControls.svelte     # кнопки окна
│   │   └── Background.svelte         # фон
│   ├── stores/                   # состояние (Svelte 5)
│   │   ├── serverStore.svelte.ts     # состояние сервера
│   │   ├── fivemStore.svelte.ts      # интеграция с FiveM
│   │   ├── storeStore.svelte.ts      # состояние магазина
│   │   ├── updateStore.svelte.ts     # логика обновлений
│   │   ├── onboardingStore.svelte.ts # онбординг
│   │   └── audioStore.svelte.ts      # звук
│   ├── App.svelte
│   ├── LauncherFrame.svelte
│   ├── i18n.ts                   # локализация
│   ├── constants.ts
│   ├── errorHandling.ts
│   ├── types.ts
│   └── main.js
├── src-tauri/                    # нативное ядро на Rust
│   ├── src/
│   ├── capabilities/             # разрешения Tauri
│   ├── icons/
│   ├── Cargo.toml
│   ├── build.rs
│   └── tauri.conf.json
├── assets/                       # логотипы
├── public/
├── scripts/
├── dist/                         # релизные артефакты
│   ├── latest.json               # манифест автообновления
│   ├── latest.json.sig           # подпись манифеста
│   └── latest-1.7.0.json
├── splash.html
├── svelte.config.js
├── tailwind.config.js
├── vite.config.js
└── postcss.config.js
```

## 📜 Скрипты

| Команда | Что делает |
|---------|-----------|
| `npm run dev` | Запуск интерфейса в браузере (Vite) |
| `npm run build` | Сборка фронтенда |
| `npm run preview` | Предпросмотр сборки |
| `npm run tauri` | Команды Tauri CLI (сборка и запуск десктоп-приложения) |
| `npm run lint` | ESLint по `.js`, `.ts`, `.svelte` |
| `npm run lint:fix` | ESLint с автоисправлением |
| `npm run format` | Форматирование Prettier |

## 🚀 Сборка и запуск

```bash
# зависимости
npm install

# разработка интерфейса (браузер)
npm run dev

# запуск десктоп-приложения в режиме разработки
npm run tauri dev

# сборка установщика
npm run tauri build
```

> Для сборки десктоп-версии нужен установленный **Rust** и зависимости Tauri 2.

## 🔄 Как работает автообновление

1. Приложение запрашивает манифест `latest.json` с сервера релизов
2. Сравнивает версию с текущей и показывает индикатор, если есть новая
3. Пользователь подтверждает обновление в модальном окне
4. Скачанный пакет проверяется **цифровой подписью** (`latest.json.sig`) — подмена файла не пройдёт
5. Приложение обновляется и перезапускается

## 🧠 Что демонстрирует проект

- **Десктопная разработка, а не веб:** Tauri 2 + Rust, работа с IPC, разрешениями и нативными плагинами
- **Разделение состояния** на шесть независимых Svelte-stores вместо одного «божественного» объекта
- **Безопасные обновления** с проверкой подписи — на уровне практик, а не «скачали и заменили»
- **Локализация и онбординг** — забота о пользователе, а не только о функциональности
- **Сборка под Windows** с установщиком и релизными артефактами

## 🔮 Планы развития

- Автоматизация релизов через GitHub Actions
- Поддержка macOS / Linux
- Тёмная и светлая темы интерфейса

## 👤 Автор

Иван Соломин — [github.com/Districkov](https://github.com/Districkov)
