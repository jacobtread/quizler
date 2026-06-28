import { type Plugin } from "vite";
import fs from "node:fs/promises";

/**
 * Simple vite plugin that creates svelte components from svg files imported using .svg?component
 */
export default function svelteSvgComponent(): Plugin {
  return {
    name: "svelte-svg-component",
    enforce: "pre",

    async resolveId(id) {
      if (id.includes(".svg?component")) {
        const realPath = id.replace(".svg?component", ".svg");
        return {
          id: realPath.replace(".svg", ".svg.svelte?svg-component"),
          meta: { realPath: realPath }
        };
      }
    },

    async load(id) {
      if (id.includes(".svg.svelte?svg-component")) {
        const moduleInfo = this.getModuleInfo(id);
        const realPath = moduleInfo?.meta?.realPath;
        if (realPath) {
          const src = await fs.readFile(realPath, "utf-8");
          // Clean out any XML header junk
          let svgRaw = src.replace(/<\?xml.*?\?>/, "");
          // Append component props to the end of the svg tag
          svgRaw = svgRaw.replace(/(<svg[^>]*?)(>)/i, "$1 {...props}$2");

          return `
<script lang="ts">
import type { SVGAttributes } from 'svelte/elements';
let props: SVGAttributes<SVGSVGElement> = $props();
</script>
${svgRaw}
`;
        }
      }
    }
  };
}
