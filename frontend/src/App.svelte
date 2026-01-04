<script lang="ts">
  import DirectoryList from "$lib/components/DirectoryList.svelte";
  import AddDirectory from "$lib/components/AddDirectory.svelte";
  import { locale, t } from "svelte-i18n";
  import { Button } from "$lib/components/ui/button";

  let refreshTrigger = 0;

  function handleAdded() {
    refreshTrigger++;
  }

  function toggleLanguage() {
    locale.update((l) => (l === "zh-CN" ? "en" : "zh-CN"));
  }
</script>

<div class="container mx-auto py-8">
  <div class="flex justify-between items-center mb-8">
    <h1 class="text-3xl font-bold">{$t("app.title")}</h1>
    <Button variant="outline" size="sm" onclick={toggleLanguage}>
      {$locale === "zh-CN" ? "English" : "中文"}
    </Button>
  </div>

  <AddDirectory onadded={handleAdded} />
  <DirectoryList {refreshTrigger} />
</div>
