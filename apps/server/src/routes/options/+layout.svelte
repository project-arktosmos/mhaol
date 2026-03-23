<script lang="ts">
  import { page } from "$app/stores";
  import classNames from "classnames";
  import { sections } from "./sections";

  let { children } = $props();

  let activeId = $derived($page.params.section || sections[0].id);
</script>

<div class="flex min-h-0 flex-1">
  <ul
    class={classNames(
      "menu hidden w-52 shrink-0 gap-1 border-r border-base-300 bg-base-200 pt-4 lg:flex",
    )}
  >
    {#each sections as section}
      <li>
        <a
          href="/options/{section.id}"
          class={classNames({ active: section.id === activeId })}
        >
          {section.label}
        </a>
      </li>
    {/each}
  </ul>

  <div class="absolute left-4 top-14 z-10 lg:hidden">
    <select
      class="select-bordered select select-sm"
      value={activeId}
      onchange={(e) => {
        window.location.href = `/options/${e.currentTarget.value}`;
      }}
    >
      {#each sections as section}
        <option value={section.id}>{section.label}</option>
      {/each}
    </select>
  </div>

  <div class="min-w-0 flex-1 overflow-y-auto p-6 pt-12 lg:pt-6">
    {@render children?.()}
  </div>
</div>
