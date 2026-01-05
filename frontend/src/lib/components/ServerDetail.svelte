<script lang="ts">
    import {
        Tabs,
        TabsContent,
        TabsList,
        TabsTrigger,
    } from "$lib/components/ui/tabs";
    import { Button } from "$lib/components/ui/button";
    import { ArrowLeft, RefreshCw } from "lucide-svelte";
    import ContainerList from "./ContainerList.svelte";
    import { getServerStatus, getServer } from "$lib/api";
    import { onMount } from "svelte";
    import {
        Card,
        CardContent,
        CardHeader,
        CardTitle,
    } from "$lib/components/ui/card";
    import { t } from "svelte-i18n";

    let { server, onback }: { server: any; onback: () => void } = $props();

    let overviewData: any = $state(null);

    onMount(async () => {
        try {
            if (server.name === "Loading...") {
                server = await getServer(server.id);
            }
            overviewData = await getServerStatus(server.id);
        } catch (e) {
            console.error("Failed to load overview", e);
        }
    });

    function formatBytes(bytes: number, decimals = 2) {
        if (!+bytes) return "0 Bytes";
        const k = 1024;
        const dm = decimals < 0 ? 0 : decimals;
        const sizes = ["Bytes", "KB", "MB", "GB", "TB", "PB", "EB", "ZB", "YB"];
        const i = Math.floor(Math.log(bytes) / Math.log(k));
        return `${parseFloat((bytes / Math.pow(k, i)).toFixed(dm))} ${sizes[i]}`;
    }

    let currentTab = $state("overview");
    // Initialize from URL immediately
    const params = new URLSearchParams(window.location.search);
    const tabParam = params.get("tab");
    if (tabParam && ["overview", "containers"].includes(tabParam)) {
        currentTab = tabParam;
    }
    let containerListComp: any = $state(null);
    let containerLoading = $state(false);
    let overviewLoading = $state(false);

    async function refreshOverview() {
        overviewLoading = true;
        try {
            overviewData = await getServerStatus(server.id);
        } catch (e) {
            console.error("Failed to refresh overview", e);
        } finally {
            overviewLoading = false;
        }
    }

    function handleRefresh() {
        if (currentTab === "overview") {
            refreshOverview();
        } else if (currentTab === "containers") {
            containerListComp?.refresh();
        }
    }

    let isRefreshing = $derived(
        currentTab === "overview" ? overviewLoading : containerLoading,
    );

    $effect(() => {
        const url = new URL(window.location.href);
        url.searchParams.set("tab", currentTab);
        window.history.replaceState({}, "", url);
    });

    // Display Logic for Server Name
    let displayServerName = $derived(server.name);
</script>

<div class="space-y-6">
    <div class="flex items-center space-x-4">
        <Button variant="ghost" size="icon" onclick={onback}>
            <ArrowLeft class="h-5 w-5" />
        </Button>
        <div class="flex flex-col">
            <h2 class="text-2xl font-bold tracking-tight">
                {displayServerName}
            </h2>
            {#if overviewData || server.name === "Loading..."}
                <span class="text-xs text-muted-foreground"
                    >{server.host || "..."}</span
                >
            {:else}
                <span class="text-xs text-muted-foreground">{server.host}</span>
            {/if}
        </div>
    </div>

    <!-- @ts-ignore -->
    <Tabs bind:value={currentTab} class="w-full">
        <div class="flex justify-between items-center mb-2">
            <TabsList>
                <TabsTrigger value="overview"
                    >{$t("servers.detail.overview")}</TabsTrigger
                >
                <TabsTrigger value="containers"
                    >{$t("servers.detail.containers")}</TabsTrigger
                >
            </TabsList>
            <Button
                variant="outline"
                size="sm"
                onclick={handleRefresh}
                disabled={isRefreshing}
            >
                <RefreshCw
                    class="h-4 w-4 mr-2 {isRefreshing ? 'animate-spin' : ''}"
                />
                {$t("servers.container_list.refresh")}
            </Button>
        </div>
        <TabsContent value="overview">
            {#if overviewData && overviewData.data}
                <div class="grid grid-cols-1 md:grid-cols-2 gap-4 mt-4">
                    <Card>
                        <CardHeader>
                            <CardTitle
                                >{$t("servers.detail.system_info")}</CardTitle
                            >
                        </CardHeader>
                        <CardContent class="grid gap-2">
                            <div class="flex justify-between">
                                <span class="text-muted-foreground"
                                    >{$t("servers.detail.os")}:</span
                                >
                                <span class="font-medium capitalize"
                                    >{overviewData.data.os}</span
                                >
                            </div>
                            <div class="flex justify-between">
                                <span class="text-muted-foreground"
                                    >{$t("servers.detail.platform")}:</span
                                >
                                <span class="font-medium capitalize"
                                    >{overviewData.data.platform} / {overviewData
                                        .data.platformFamily}</span
                                >
                            </div>
                            <div class="flex justify-between">
                                <span class="text-muted-foreground"
                                    >{$t("servers.detail.kernel")}:</span
                                >
                                <span class="font-medium"
                                    >{overviewData.data.kernelVersion} ({overviewData
                                        .data.kernelArch})</span
                                >
                            </div>
                        </CardContent>
                    </Card>
                    <Card>
                        <CardHeader>
                            <CardTitle
                                >{$t("servers.detail.resources")}</CardTitle
                            >
                        </CardHeader>
                        <CardContent>
                            <div class="flex justify-between">
                                <span class="text-muted-foreground"
                                    >{$t(
                                        "servers.detail.disk_available",
                                    )}:</span
                                >
                                <span class="font-medium"
                                    >{formatBytes(
                                        overviewData.data.diskSize,
                                    )}</span
                                >
                            </div>
                        </CardContent>
                    </Card>
                </div>
            {:else if overviewData}
                <div class="p-4 border rounded-md text-red-500 mt-4">
                    {$t("servers.detail.error_loading")}: {overviewData.message ||
                        "Unknown error"}
                </div>
            {:else}
                <div class="p-4 text-muted-foreground mt-4">
                    {$t("picker.loading")}...
                </div>
            {/if}
        </TabsContent>
        <TabsContent value="containers">
            <!-- @ts-ignore -->
            <ContainerList
                serverId={server.id}
                bind:this={containerListComp}
                bind:loading={containerLoading}
            />
        </TabsContent>
    </Tabs>
</div>
