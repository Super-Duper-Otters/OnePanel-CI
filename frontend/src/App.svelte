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
  import ServerDetail from "$lib/components/ServerDetail.svelte";
  import DockerDashboard from "$lib/components/DockerDashboard.svelte";
  import { Settings, Plus } from "lucide-svelte";
  import SettingsDialog from "$lib/components/SettingsDialog.svelte";
  import AddDirectoryDialog from "$lib/components/AddDirectoryDialog.svelte";
  import AddServerDialog from "$lib/components/AddServerDialog.svelte";
  import { getVersion } from "$lib/api";

  let settingsOpen = $state(false);
  let addRepoOpen = $state(false);
  let addServerOpen = $state(false);
  let currentTab = $state("repositories");
  let appVersion = $state("");

  onMount(async () => {
    const params = new URLSearchParams(window.location.search);
    const view = params.get("view");
    const path = params.get("path");
    const sId = params.get("serverId");

    if (view) currentTab = view;
    if (path) selectedPath = path;
    if (sId) {
      selectedServer = { id: parseInt(sId), name: "Loading..." };
    }

    try {
      const v = await getVersion();
      appVersion = v.version;
    } catch (e) {
      console.error("Failed to fetch version", e);
    }
  });

  $effect(() => {
    const url = new URL(window.location.href);
    url.searchParams.set("view", currentTab);

    if (selectedPath) {
      url.searchParams.set("path", selectedPath);
    } else {
      url.searchParams.delete("path");
    }

    if (selectedServer) {
      url.searchParams.set("serverId", selectedServer.id.toString());
    } else {
      url.searchParams.delete("serverId");
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

  let selectedServer = $state<any>(null);

  function handleServerSelect(server: any) {
    selectedServer = server;
  }

  function handleServerBack() {
    selectedServer = null;
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
      <div class="flex items-center gap-3">
        <h1 class="text-3xl font-bold">{$t("app.title")}</h1>
        {#if appVersion}
          <div
            class="inline-flex items-center rounded-full border px-2.5 py-0.5 text-xs font-semibold text-muted-foreground"
          >
            v{appVersion}
          </div>
        {/if}
      </div>
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
        {#if selectedServer}
          <!-- @ts-ignore -->
          <ServerDetail server={selectedServer} onback={handleServerBack} />
        {:else}
          <!-- @ts-ignore -->
          <ServerList bind:this={serverList} onselect={handleServerSelect} />
        {/if}
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
