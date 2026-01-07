<script lang="ts">
    import { onMount } from "svelte";
    import { t } from "svelte-i18n";
    import {
        Card,
        CardContent,
        CardHeader,
        CardTitle,
        CardDescription,
    } from "$lib/components/ui/card";
    import { Button } from "$lib/components/ui/button";
    import { Badge } from "$lib/components/ui/badge";
    import {
        Loader2,
        Play,
        Square,
        Trash2,
        FileText,
        RefreshCw,
    } from "lucide-svelte";
    import * as Dialog from "$lib/components/ui/dialog";
    import { ScrollArea } from "$lib/components/ui/scroll-area";

    interface ContainerSummary {
        id: string;
        names: string[];
        image: string;
        state: string;
        status: string;
    }

    let containers = $state<ContainerSummary[]>([]);
    let loading = $state(false);
    let operationLoading = $state<string | null>(null); // Stores ID of container currently being operated on
    let logsOpen = $state(false);
    let currentLogs = $state("");
    let logLoading = $state(false);
    let currentContainerId = $state("");

    async function fetchContainers() {
        loading = true;
        try {
            const res = await fetch(
                "http://localhost:3000/api/docker/containers",
            );
            if (res.ok) {
                containers = await res.json();
            }
        } catch (error) {
            console.error(error);
        } finally {
            loading = false;
        }
    }

    async function operateContainer(
        id: string,
        action: "start" | "stop" | "remove",
    ) {
        operationLoading = id;
        try {
            let method = "POST";
            let url = `http://localhost:3000/api/docker/containers/${id}/${action}`;
            if (action === "remove") {
                method = "DELETE";
                url = `http://localhost:3000/api/docker/containers/${id}`;
            }

            const res = await fetch(url, { method });
            if (res.ok) {
                await fetchContainers();
            } else {
                alert(
                    $t("docker.container_table.operation_failed") ||
                        "Operation failed",
                );
            }
        } catch (error) {
            console.error(error);
        } finally {
            operationLoading = null;
        }
    }

    async function viewLogs(id: string) {
        currentContainerId = id;
        logsOpen = true;
        logLoading = true;
        currentLogs = "";
        try {
            const res = await fetch(
                `http://localhost:3000/api/docker/containers/${id}/logs`,
            );
            if (res.ok) {
                currentLogs = await res.text();
            }
        } catch (error) {
            console.error(error);
            currentLogs =
                $t("docker.container_table.failed_logs") ||
                "Failed to load logs";
        } finally {
            logLoading = false;
        }
    }

    onMount(() => {
        fetchContainers();
    });
</script>

<div class="space-y-4">
    {#if loading && containers.length === 0}
        <div class="text-center py-10 text-muted-foreground">
            <Loader2 class="animate-spin mx-auto" />
        </div>
    {:else if containers.length === 0}
        <div class="text-center py-10 text-muted-foreground">
            {$t("docker.container_table.no_containers") ||
                "No containers found"}
        </div>
    {:else}
        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
            {#each containers as container}
                <Card>
                    <CardHeader
                        class="flex flex-row items-center justify-between space-y-0 pb-2"
                    >
                        <div class="space-y-1 overflow-hidden mr-2">
                            <CardTitle title={container.names.join(", ")}>
                                {container.names
                                    .map((n) => n.replace(/^\//, ""))
                                    .join(", ")}
                            </CardTitle>
                            <CardDescription
                                class="truncate"
                                title={container.image}
                            >
                                {container.image}
                            </CardDescription>
                        </div>
                        <div class="flex gap-1 shrink-0">
                            {#if container.state !== "running"}
                                <Button
                                    variant="ghost"
                                    size="icon"
                                    title="Start"
                                    disabled={operationLoading === container.id}
                                    onclick={() =>
                                        operateContainer(container.id, "start")}
                                >
                                    {#if operationLoading === container.id}
                                        <Loader2
                                            class="animate-spin"
                                            size={16}
                                        />
                                    {:else}
                                        <Play
                                            size={16}
                                            class="text-green-500"
                                        />
                                    {/if}
                                </Button>
                            {/if}
                            {#if container.state === "running"}
                                <Button
                                    variant="ghost"
                                    size="icon"
                                    title="Stop"
                                    disabled={operationLoading === container.id}
                                    onclick={() =>
                                        operateContainer(container.id, "stop")}
                                >
                                    {#if operationLoading === container.id}
                                        <Loader2
                                            class="animate-spin"
                                            size={16}
                                        />
                                    {:else}
                                        <Square
                                            size={16}
                                            class="text-red-500"
                                        />
                                    {/if}
                                </Button>
                            {/if}
                            <Button
                                variant="ghost"
                                size="icon"
                                title="Logs"
                                onclick={() => viewLogs(container.id)}
                            >
                                <FileText size={16} />
                            </Button>
                            <Button
                                variant="ghost"
                                size="icon"
                                class="text-red-500 hover:text-red-700 hover:bg-red-50"
                                title="Remove"
                                disabled={operationLoading === container.id}
                                onclick={() => {
                                    if (
                                        confirm(
                                            $t(
                                                "docker.container_table.confirm_remove",
                                            ) || "Are you sure?",
                                        )
                                    ) {
                                        operateContainer(
                                            container.id,
                                            "remove",
                                        );
                                    }
                                }}
                            >
                                <Trash2 size={16} />
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
                                <span
                                    class="text-xs text-muted-foreground font-mono"
                                    >{container.id.substring(0, 12)}</span
                                >
                            </div>
                            <div
                                class="text-xs text-muted-foreground truncate"
                                title={container.status}
                            >
                                {container.status}
                            </div>
                        </div>
                    </CardContent>
                </Card>
            {/each}
        </div>
    {/if}

    <Dialog.Root bind:open={logsOpen}>
        <Dialog.Content class="max-w-4xl max-h-[80vh] flex flex-col">
            <Dialog.Header>
                <Dialog.Title
                    >Container Logs ({currentContainerId})</Dialog.Title
                >
            </Dialog.Header>
            <div
                class="flex-1 overflow-hidden rounded-md bg-zinc-950 p-4 text-xs font-mono text-zinc-50"
            >
                <ScrollArea class="h-[60vh]">
                    {#if logLoading}
                        <div class="flex justify-center items-center h-full">
                            <Loader2 class="animate-spin" />
                        </div>
                    {:else}
                        <pre class="whitespace-pre-wrap">{currentLogs ||
                                $t("docker.container_table.no_logs") ||
                                "No logs available"}</pre>
                    {/if}
                </ScrollArea>
            </div>
        </Dialog.Content>
    </Dialog.Root>
</div>
