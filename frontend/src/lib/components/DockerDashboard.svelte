<script lang="ts">
    import { onMount, onDestroy } from "svelte";
    import { t } from "svelte-i18n";
    import {
        Card,
        CardContent,
        CardHeader,
        CardTitle,
    } from "$lib/components/ui/card";
    import { Button } from "$lib/components/ui/button";
    import {
        RefreshCw,
        Container,
        Box,
        ArrowLeft,
        Download,
    } from "lucide-svelte";
    import { Tabs, TabsContent } from "$lib/components/ui/tabs";
    import DockerContainers from "./DockerContainers.svelte";
    import DockerImages from "./DockerImages.svelte";

    interface DockerInfo {
        version: string;
        containers: number;
        images: number;
        status: string;
    }

    let dockerInfo = $state<DockerInfo | null>(null);
    let loading = $state(false);
    let error = $state<string | null>(null);
    let currentTab = $state("dashboard");
    let dockerImagesComponent = $state<any>(null);
    let interval: any;

    async function fetchDockerInfo() {
        loading = true;
        error = null;
        try {
            const res = await fetch("http://localhost:3000/api/docker/info");
            if (res.ok) {
                dockerInfo = await res.json();
            } else {
                error = "Failed to connect to Docker";
            }
        } catch (e) {
            error = "Error connecting to backend";
        } finally {
            loading = false;
        }
    }

    onMount(() => {
        fetchDockerInfo();
    });

    onDestroy(() => {
        if (interval) clearInterval(interval);
    });
</script>

<div class="h-full flex flex-col space-y-6">
    {#if error}
        <div
            class="text-red-500 bg-red-50 p-4 rounded-md border border-red-200"
        >
            {error}. Is Docker Desktop running?
        </div>
    {/if}

    <Tabs bind:value={currentTab} class="w-full">
        <TabsContent value="dashboard" class="space-y-4">
            <div class="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
                <Card>
                    <CardHeader
                        class="flex flex-row items-center justify-between space-y-0 pb-2"
                    >
                        <CardTitle class="text-sm font-medium"
                            >{$t("docker.status") || "Status"}</CardTitle
                        >
                        <div class="h-4 w-4 text-muted-foreground">
                            <div
                                class={dockerInfo
                                    ? "w-3 h-3 rounded-full bg-green-500"
                                    : "w-3 h-3 rounded-full bg-red-500"}
                            ></div>
                        </div>
                    </CardHeader>
                    <CardContent>
                        <div class="text-2xl font-bold">
                            {dockerInfo ? "Connected" : "Disconnected"}
                        </div>
                        <p class="text-xs text-muted-foreground">
                            {dockerInfo?.version
                                ? `Version: ${dockerInfo.version}`
                                : "-"}
                        </p>
                    </CardContent>
                </Card>

                <Card
                    class="cursor-pointer hover:bg-muted/50 transition-colors"
                    onclick={() => (currentTab = "containers")}
                >
                    <CardHeader
                        class="flex flex-row items-center justify-between space-y-0 pb-2"
                    >
                        <CardTitle class="text-sm font-medium"
                            >{$t("docker.containers") ||
                                "Containers"}</CardTitle
                        >
                        <Container class="h-4 w-4 text-muted-foreground" />
                    </CardHeader>
                    <CardContent>
                        <div class="text-2xl font-bold">
                            {dockerInfo?.containers || 0}
                        </div>
                    </CardContent>
                </Card>

                <Card
                    class="cursor-pointer hover:bg-muted/50 transition-colors"
                    onclick={() => (currentTab = "images")}
                >
                    <CardHeader
                        class="flex flex-row items-center justify-between space-y-0 pb-2"
                    >
                        <CardTitle class="text-sm font-medium"
                            >{$t("docker.images") || "Images"}</CardTitle
                        >
                        <Box class="h-4 w-4 text-muted-foreground" />
                    </CardHeader>
                    <CardContent>
                        <div class="text-2xl font-bold">
                            {dockerInfo?.images || 0}
                        </div>
                    </CardContent>
                </Card>
            </div>

            <div class="grid gap-4 mt-4">
                <Card class="bg-mutedAPI">
                    <CardHeader>
                        <CardTitle>Feature Coming Soon</CardTitle>
                    </CardHeader>
                    <CardContent>
                        Building images and exporting to 1Panel will be
                        supported here.
                    </CardContent>
                </Card>
            </div>
        </TabsContent>
        <TabsContent value="containers">
            <div class="flex items-center mb-4">
                <Button
                    variant="ghost"
                    size="sm"
                    onclick={() => (currentTab = "dashboard")}
                    class="mr-4"
                >
                    <ArrowLeft class="mr-2" size={16} />
                    {$t("common.back") || "Back"}
                </Button>
                <h3 class="text-lg font-medium">
                    {$t("docker.containers") || "Containers"}
                </h3>
            </div>
            <DockerContainers />
        </TabsContent>
        <TabsContent value="images">
            <div class="flex items-center justify-between mb-4">
                <div class="flex items-center">
                    <Button
                        variant="ghost"
                        size="sm"
                        onclick={() => (currentTab = "dashboard")}
                        class="mr-4"
                    >
                        <ArrowLeft class="mr-2" size={16} />
                        {$t("common.back") || "Back"}
                    </Button>
                    <h3 class="text-lg font-medium">
                        {$t("docker.images_panel.title") || "Docker Images"}
                    </h3>
                </div>
                <Button
                    variant="default"
                    size="sm"
                    onclick={() => dockerImagesComponent?.openPullDialog()}
                >
                    <Download class="mr-2" size={16} />
                    {$t("docker.images_panel.pull_image") || "Pull Image"}
                </Button>
            </div>
            <DockerImages bind:this={dockerImagesComponent} />
        </TabsContent>
    </Tabs>
</div>
