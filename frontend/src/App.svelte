<script lang="ts">
  import { onMount } from "svelte";
  import { locale, t, isLoading } from "svelte-i18n";
  import { Button } from "$lib/components/ui/button";
  import { Toaster } from "$lib/components/ui/sonner";
  import {
    Tabs,
    TabsContent,
    TabsList,
    TabsTrigger,
  } from "$lib/components/ui/tabs";
  import DirectoryList from "$lib/components/DirectoryList.svelte";
  import DirectoryDetail from "$lib/components/DirectoryDetail.svelte";
  import ServerList from "$lib/components/ServerList.svelte";
  import DockerDashboard from "$lib/components/DockerDashboard.svelte";
  import { Settings, Plus } from "lucide-svelte";
  import SettingsDialog from "$lib/components/SettingsDialog.svelte";
  import AddDirectoryDialog from "$lib/components/AddDirectoryDialog.svelte";
  import AddServerDialog from "$lib/components/AddServerDialog.svelte";

  let settingsOpen = $state(false);
  let addRepoOpen = $state(false);
  let addServerOpen = $state(false);
  let currentTab = $state("repositories");

  onMount(() => {
    const params = new URLSearchParams(window.location.search);
    const tab = params.get("tab");
    const path = params.get("path");
    if (tab) currentTab = tab;
    if (path) selectedPath = path;
  });

  $effect(() => {
    const url = new URL(window.location.href);
    url.searchParams.set("tab", currentTab);
    if (selectedPath) {
      url.searchParams.set("path", selectedPath);
    } else {
      url.searchParams.delete("path");
    }
    window.history.replaceState({}, "", url);
  });

  // Repository list state
  let refreshTrigger = $state(0);
  let selectedPath = $state<string | null>(null);
  let serverList = $state<any>(null); // For server list refresh if needed

  function handleSelect(path: string) {
    selectedPath = path;
  }

  function handleBack() {
    selectedPath = null;
  }

  function onRepoAdded() {
    refreshTrigger++;
  }

  function onServerAdded() {
    // serverList might auto-refresh or need trigger
    if (serverList && serverList.refresh) {
      serverList.refresh();
    }
  }

  const onAdd = () => {
    if (currentTab === "repositories") {
      addRepoOpen = true;
    } else if (currentTab === "servers") {
      addServerOpen = true;
    }
  };
</script>

{#if $isLoading}
  <div class="flex items-center justify-center min-h-screen">Loading...</div>
{:else}
  <Toaster />
  <main class="container mx-auto py-8">
    <div class="flex justify-between items-center mb-8">
      <h1 class="text-3xl font-bold">{$t("app.title")}</h1>
      <Button variant="ghost" size="icon" onclick={() => (settingsOpen = true)}>
        <Settings class="h-5 w-5" />
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
          <TabsTrigger value="docker"
            >{$t("tabs.docker") || "Docker"}</TabsTrigger
          >
        </TabsList>
        <div>
          {#if currentTab !== "docker" && (currentTab !== "repositories" || !selectedPath)}
            <Button onclick={onAdd}>
              <Plus class="mr-2 h-4 w-4" />
              {$t("directory.add_button")}
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
      <TabsContent value="docker">
        <DockerDashboard />
      </TabsContent>
    </Tabs>

    <SettingsDialog bind:open={settingsOpen} />
    <AddDirectoryDialog bind:open={addRepoOpen} onadded={onRepoAdded} />
    <AddServerDialog bind:open={addServerOpen} onadded={onServerAdded} />
  </main>
{/if}
