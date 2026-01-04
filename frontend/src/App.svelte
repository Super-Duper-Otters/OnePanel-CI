<script lang="ts">
  import DirectoryList from "$lib/components/DirectoryList.svelte";
  import AddDirectory from "$lib/components/AddDirectory.svelte";

  let refreshTrigger = 0;

  function handleAdded() {
    refreshTrigger++;
  }
</script>

<div class="container mx-auto py-8">
  <h1 class="text-3xl font-bold mb-8">OnePanel CI/CD Manager</h1>

  <AddDirectory onadded={handleAdded} />
  <DirectoryList {refreshTrigger} />
</div>
