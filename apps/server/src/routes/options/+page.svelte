<script lang="ts">
  import { goto } from "$app/navigation";
  import { onMount } from "svelte";
  import { sections } from "./sections";

  onMount(() => {
    goto(`/options/${sections[0].id}`, { replaceState: true });
  });
</script>
