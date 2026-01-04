<script lang="ts">
  import DirectoryList from "$lib/components/DirectoryList.svelte";
  import AddDirectory from "$lib/components/AddDirectory.svelte";
  import DirectoryDetail from "$lib/components/DirectoryDetail.svelte";
  import { locale, t, isLoading } from "svelte-i18n";
  import { Button } from "$lib/components/ui/button";

  let refreshTrigger = $state(0);
  let selectedPath = $state<string | null>(null);

  function handleAdded() {
    refreshTrigger++;
  }

  function handleSelect(path: string) {
    selectedPath = path;
  }

  function handleBack() {
    selectedPath = null;
  }

  function toggleLanguage() {
    locale.update((l) => (l === "zh-CN" ? "en" : "zh-CN"));
  }
</script>

{#if $isLoading}
  <div class="flex items-center justify-center min-h-screen">Loading...</div>
{:else}
  <div class="container mx-auto py-8">
    <div class="flex justify-between items-center mb-8">
      <h1 class="text-3xl font-bold">{$t("app.title")}</h1>
      <Button variant="outline" size="sm" onclick={toggleLanguage}>
        {$locale === "zh-CN" ? "English" : "中文"}
      </Button>
    </div>

    {#if selectedPath}
      <DirectoryDetail path={selectedPath} onback={handleBack} />
    {:else}
      <AddDirectory onadded={handleAdded} />
      <DirectoryList {refreshTrigger} onselect={handleSelect} />
    {/if}
  </div>
{/if}
