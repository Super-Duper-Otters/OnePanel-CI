<script lang="ts">
    import { onMount } from "svelte";
    import {
        listContainers,
        operateContainer,
        getContainerLogs,
    } from "$lib/api";
    import type { Container } from "$lib/api";
    import { Button } from "$lib/components/ui/button";
    import {
        Card,
        CardContent,
        CardHeader,
        CardTitle,
        CardDescription,
    } from "$lib/components/ui/card";
    import { RefreshCw, Play, Square, RotateCw, FileText } from "lucide-svelte";
    import { toast } from "svelte-sonner";
    import {
        Dialog,
        DialogContent,
        DialogHeader,
        DialogTitle,
    } from "$lib/components/ui/dialog";
    import { Badge } from "$lib/components/ui/badge";
    import { t } from "svelte-i18n";

    let {
        serverId,
        loading = $bindable(false),
    }: { serverId: number; loading?: boolean } = $props();
    // We might want to pass these prompts in or use i18n
    // For now hardcoded English/Chinese based on context or simple English

    let containers = $state<Container[]>([]);

    // Log viewer state
    let logOpen = $state(false);
    let logContent = $state("");
    let logTitle = $state("");
    let logLoading = $state(false);

    export async function refresh() {
        if (!serverId) return;
        loading = true;
        try {
            containers = await listContainers(serverId);
        } catch (e) {
            toast.error("Failed to load containers");
            console.error(e);
        } finally {
            loading = false;
        }
    }

    // Alias for internal calls if any (though we removed the button)
    const load = refresh;

    async function handleOperate(name: string, op: string) {
        try {
            toast.info(`Sending ${op} signal to ${name}...`);
            await operateContainer(serverId, [name], op);
            toast.success(`Successfully sent ${op} to ${name}`);
            await refresh();
        } catch (e) {
            toast.error(`Failed to ${op} ${name}`);
        }
    }

    async function handleLogs(name: string) {
        logTitle = `Logs: ${name}`;
        logOpen = true;
        logLoading = true;
        logContent = "";
        try {
            logContent = await getContainerLogs(serverId, name);
        } catch (e) {
            logContent = "Failed to load logs.";
            toast.error("Failed to load logs");
        } finally {
            logLoading = false;
        }
    }

    onMount(() => {
        refresh();
    });
</script>

<div class="space-y-4">
    {#if loading && containers.length === 0}
        <div class="text-center py-10 text-muted-foreground">
            {$t("servers.container_list.loading")}
        </div>
    {:else if containers.length === 0}
        <div class="text-center py-10 text-muted-foreground">
            {$t("servers.container_list.no_containers")}
        </div>
    {:else}
        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
            {#each containers as container}
                <Card>
                    <CardHeader
                        class="flex flex-row items-center justify-between space-y-0 pb-2"
                    >
                        <div class="space-y-1 overflow-hidden mr-2">
                            <CardTitle title={container.name}>
                                {container.name.replace(/^\//, "")}
                            </CardTitle>
                            <CardDescription
                                class="truncate"
                                title={container.imageName}
                            >
                                {container.imageName}
                            </CardDescription>
                        </div>
                        <div class="flex gap-1 shrink-0">
                            {#if container.state !== "running"}
                                <Button
                                    variant="ghost"
                                    size="icon"
                                    title={$t(
                                        "servers.container_list.actions.start",
                                    )}
                                    onclick={() =>
                                        handleOperate(container.name, "start")}
                                >
                                    <Play class="h-4 w-4 text-green-500" />
                                </Button>
                            {/if}
                            <Button
                                variant="ghost"
                                size="icon"
                                title={$t(
                                    "servers.container_list.actions.restart",
                                )}
                                onclick={() =>
                                    handleOperate(container.name, "restart")}
                            >
                                <RotateCw class="h-4 w-4 text-blue-500" />
                            </Button>
                            {#if container.state === "running"}
                                <Button
                                    variant="ghost"
                                    size="icon"
                                    title={$t(
                                        "servers.container_list.actions.stop",
                                    )}
                                    onclick={() =>
                                        handleOperate(container.name, "stop")}
                                >
                                    <Square class="h-4 w-4 text-red-500" />
                                </Button>
                            {/if}
                            <Button
                                variant="ghost"
                                size="icon"
                                title={$t(
                                    "servers.container_list.actions.logs",
                                )}
                                onclick={() => handleLogs(container.name)}
                            >
                                <FileText class="h-4 w-4" />
                            </Button>
                        </div>
                    </CardHeader>
                    <CardContent>
                        <div class="flex flex-col gap-2">
                            <div class="flex items-center gap-2">
                                <Badge
                                    variant="outline"
                                    class={container.state === "running"
                                        ? "bg-green-100 text-green-800 border-green-200"
                                        : "bg-gray-100 text-gray-800 border-gray-200"}
                                >
                                    {container.state}
                                </Badge>
                            </div>
                            {#if container.ports && container.ports.length > 0}
                                <div class="flex flex-wrap gap-1">
                                    {#each container.ports.slice(0, 3) as port}
                                        <Badge
                                            variant="secondary"
                                            class="font-mono text-xs"
                                        >
                                            {port}
                                        </Badge>
                                    {/each}
                                    {#if container.ports.length > 3}
                                        <Badge
                                            variant="secondary"
                                            class="font-mono text-xs"
                                        >
                                            +{container.ports.length - 3}
                                        </Badge>
                                    {/if}
                                </div>
                            {/if}
                        </div>
                    </CardContent>
                </Card>
            {/each}
        </div>
    {/if}

    <Dialog bind:open={logOpen}>
        <DialogContent class="max-w-4xl h-[80vh] flex flex-col">
            <DialogHeader>
                <DialogTitle>{logTitle}</DialogTitle>
            </DialogHeader>
            <div
                class="flex-1 overflow-auto bg-black text-white p-4 font-mono text-xs rounded-md whitespace-pre-wrap"
            >
                {#if logLoading}
                    Loading logs...
                {:else}
                    {logContent || "No logs available."}
                {/if}
            </div>
        </DialogContent>
    </Dialog>
</div>
