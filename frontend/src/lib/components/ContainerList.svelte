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
        Table,
        TableBody,
        TableCell,
        TableHead,
        TableHeader,
        TableRow,
    } from "$lib/components/ui/table";
    import { RefreshCw, Play, Square, RotateCw, FileText } from "lucide-svelte";
    import { toast } from "svelte-sonner";
    import {
        Dialog,
        DialogContent,
        DialogHeader,
        DialogTitle,
    } from "$lib/components/ui/dialog";
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
    <!-- Refresh button removed, handled by parent -->

    <div class="border rounded-md">
        <Table>
            <TableHeader>
                <TableRow>
                    <TableHead
                        >{$t("servers.container_list.table.status")}</TableHead
                    >
                    <TableHead
                        >{$t("servers.container_list.table.name")}</TableHead
                    >
                    <TableHead
                        >{$t("servers.container_list.table.image")}</TableHead
                    >
                    <TableHead
                        >{$t("servers.container_list.table.ports")}</TableHead
                    >
                    <TableHead class="text-right"
                        >{$t("servers.container_list.table.actions")}</TableHead
                    >
                </TableRow>
            </TableHeader>
            <TableBody>
                {#if containers.length === 0}
                    <TableRow>
                        <TableCell
                            colspan={2}
                            class="text-center h-24 text-muted-foreground"
                        >
                            {loading
                                ? $t("servers.container_list.loading")
                                : $t("servers.container_list.no_containers")}
                        </TableCell>
                    </TableRow>
                {:else}
                    {#each containers as container}
                        <TableRow>
                            <TableCell>
                                <div
                                    class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium {container.state ===
                                    'running'
                                        ? 'bg-green-100 text-green-800'
                                        : 'bg-gray-100 text-gray-800'}"
                                >
                                    {container.state}
                                </div>
                            </TableCell>
                            <TableCell class="font-medium"
                                >{container.name}</TableCell
                            >
                            <TableCell
                                class="truncate max-w-[200px]"
                                title={container.imageName}
                                >{container.imageName}</TableCell
                            >
                            <TableCell>
                                {#if container.ports && container.ports.length > 0}
                                    <div
                                        class="flex flex-col text-xs text-muted-foreground"
                                    >
                                        {#each container.ports.slice(0, 2) as port}
                                            <span>{port}</span>
                                        {/each}
                                        {#if container.ports.length > 2}
                                            <span
                                                class="text-[10px] text-gray-400"
                                                >+{container.ports.length - 2} more</span
                                            >
                                        {/if}
                                    </div>
                                {:else}
                                    -
                                {/if}
                            </TableCell>
                            <TableCell class="text-right space-x-1">
                                {#if container.state !== "running"}
                                    <Button
                                        variant="ghost"
                                        size="icon"
                                        title={$t(
                                            "servers.container_list.actions.start",
                                        )}
                                        onclick={() =>
                                            handleOperate(
                                                container.name,
                                                "start",
                                            )}
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
                                        handleOperate(
                                            container.name,
                                            "restart",
                                        )}
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
                                            handleOperate(
                                                container.name,
                                                "stop",
                                            )}
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
                            </TableCell>
                        </TableRow>
                    {/each}
                {/if}
            </TableBody>
        </Table>
    </div>

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
