import App from "./App.svelte";
import "./assets/app.css";
import { mount } from "svelte";

const app = mount(App, {
  target: document.getElementById("app") as Element
});

export default app;
