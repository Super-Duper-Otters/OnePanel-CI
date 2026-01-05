<script lang="ts">
    import { onMount, onDestroy } from "svelte";
    import { t } from "svelte-i18n";
    import {
        Card,
        CardContent,
        CardHeader,
        CardTitle,
    } from "$lib/components/ui/card";
    import { Badge } from "$lib/components/ui/badge";
    import { Button } from "$lib/components/ui/button";
    import { Loader2, RefreshCw, Container, Box } from "lucide-svelte";

    interface DockerInfo {
        version: string;
        containers: number;
        images: number;
        status: string;
    }

    let dockerInfo = $state<DockerInfo | null>(null);
    let loading = $state(false);
    let error = $state<string | null>(null);
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

<div class="container mx-auto p-4 space-y-6">
    <div class="flex items-center justify-between">
        <h2 class="text-3xl font-bold tracking-tight">
            {$t("docker.title") || "Docker Environment"}
        </h2>
        <Button
            variant="outline"
            size="sm"
            onclick={fetchDockerInfo}
            disabled={loading}
        >
            <RefreshCw
                class={loading ? "animate-spin mr-2" : "mr-2"}
                size={16}
            />
            {$t("docker.refresh") || "Refresh"}
        </Button>
    </div>

    {#if error}
        <div
            class="text-red-500 bg-red-50 p-4 rounded-md border border-red-200"
        >
            {error}. Is Docker Desktop running?
        </div>
    {/if}

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

        <Card>
            <CardHeader
                class="flex flex-row items-center justify-between space-y-0 pb-2"
            >
                <CardTitle class="text-sm font-medium"
                    >{$t("docker.containers") || "Containers"}</CardTitle
                >
                <Container class="h-4 w-4 text-muted-foreground" />
            </CardHeader>
            <CardContent>
                <div class="text-2xl font-bold">
                    {dockerInfo?.containers || 0}
                </div>
            </CardContent>
        </Card>

        <Card>
            <CardHeader
                class="flex flex-row items-center justify-between space-y-0 pb-2"
            >
                <CardTitle class="text-sm font-medium"
                    >{$t("docker.images") || "Images"}</CardTitle
                >
                <Box class="h-4 w-4 text-muted-foreground" />
            </CardHeader>
            <CardContent>
                <div class="text-2xl font-bold">{dockerInfo?.images || 0}</div>
            </CardContent>
        </Card>
    </div>

    <div class="grid gap-4 mt-4">
        <!-- Placeholder for future build features -->
        <Card class="bg-mutedAPI">
            <CardHeader>
                <CardTitle>Feature Coming Soon</CardTitle>
            </CardHeader>
            <CardContent>
                Building images and exporting to 1Panel will be supported here.
            </CardContent>
        </Card>
    </div>
</div>
