<script lang="ts">
  import { goto } from "$app/navigation";
  import { base } from "$app/paths";
  import { onMount } from "svelte";
  import { sections } from "./sections";

  onMount(() => {
    goto(`${base}/options/${sections[0].id}`, { replaceState: true });
  });
</script>
