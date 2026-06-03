// Точка входа Svelte 5-приложения
import "./app.css";
import { mount } from "svelte";
import App from "./App.svelte";

// Svelte 5: используем mount() вместо конструктора
const app = mount(App, {
  target: document.getElementById("app"),
});

export default app;
