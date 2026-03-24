<script lang="ts">
  import { page } from "$app/stores";
  import { sectionComponents } from "../sections";

  let section = $derived($page.params.section);
  let entry = $derived(sectionComponents[section]);
</script>

{#if entry}
  <entry.component {...(entry.props ?? {})} />
{:else}
  <p class="text-base-content/60">Unknown section: {section}</p>
{/if}
