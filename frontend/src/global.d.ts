declare type Item = import("svelte-dnd-action").Item;
declare type DndEvent<ItemType = Item> =
  import("svelte-dnd-action").DndEvent<ItemType>;
declare namespace svelteHTML {
  interface HTMLAttributes<T> {
    "on:finalize"?: (
      event: CustomEvent<DndEvent<ItemType>> & { target: EventTarget & T }
    ) => void;
    "on:consider"?: (
      event: CustomEvent<DndEvent<ItemType>> & { target: EventTarget & T }
    ) => void;
  }
}

declare module "*.svg?component" {
  import type { Component } from "svelte";
  import type { SVGAttributes } from "svelte/elements";

  const component: Component<SVGAttributes<SVGSVGElement>>;

  export default component;
}
