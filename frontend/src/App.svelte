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

  let settingsOpen = $state(false);
  let addRepoOpen = $state(false);
  let addServerOpen = $state(false);
  let currentTab = $state("repositories");

  onMount(() => {
    const params = new URLSearchParams(window.location.search);
    const view = params.get("view");
    const path = params.get("path");
    const sId = params.get("serverId");

    if (view) currentTab = view;
    if (path) selectedPath = path;
    if (sId) {
      // We need to restore the server object. Since we don't have the full object,
      // we might need to fetch it or finding it in the list if loaded.
      // For now, let's just assume we can pass the ID to ServerDetail or we find it in the list.
      // Actually ServerDetail takes a full object. We should probably refactor ServerDetail to take ID.
      // But to minimize changes, let's try to pass a skeleton or find it.
      // Or simpler: Just store the ID and let ServerDetail fetch if needed?
      // ServerDetail "overviewData" fetches by ID. But it displays "server.name".
      // Let's mock the server object with just ID for now, as ServerDetail mostly uses ID.
      selectedServer = { id: parseInt(sId), name: "Loading..." };
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
