<script lang="ts">
  import { locale, t, isLoading } from "svelte-i18n";
  import { Button } from "$lib/components/ui/button";
  import DirectoryList from "$lib/components/DirectoryList.svelte";
  import AddDirectory from "$lib/components/AddDirectory.svelte";
  import DirectoryDetail from "$lib/components/DirectoryDetail.svelte";
  import ServerList from "$lib/components/ServerList.svelte";
  import {
    Tabs,
    TabsContent,
    TabsList,
    TabsTrigger,
  } from "$lib/components/ui/tabs";

  import { Plus } from "lucide-svelte";

  let refreshTrigger = $state(0);
  let selectedPath = $state<string | null>(null);
  let currentTab = $state("repositories");
  let serverList = $state<any>(null);

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

    <!-- @ts-ignore -->
    <Tabs bind:value={currentTab} class="w-full">
      <div class="flex items-center justify-between mb-4">
        <TabsList>
          <TabsTrigger value="repositories"
            >{$t("tabs.repositories")}</TabsTrigger
          >
          <TabsTrigger value="servers">{$t("tabs.servers")}</TabsTrigger>
        </TabsList>
        <div>
          {#if currentTab === "repositories"}
            <AddDirectory onadded={handleAdded} />
          {:else if currentTab === "servers"}
            <Button onclick={() => serverList?.openAddDialog()}>
              <Plus class="mr-2 h-4 w-4" />
              {$t("servers.add_button")}
            </Button>
          {/if}
        </div>
      </div>

      <TabsContent value="repositories">
        {#if selectedPath}
          <!-- @ts-ignore -->
          <DirectoryDetail path={selectedPath} onback={handleBack} />
        {:else}
          <div class="space-y-4">
            <DirectoryList {refreshTrigger} onselect={handleSelect} />
          </div>
        {/if}
      </TabsContent>
      <TabsContent value="servers">
        <!-- @ts-ignore -->
        <ServerList bind:this={serverList} />
      </TabsContent>
    </Tabs>
  </div>
{/if}
