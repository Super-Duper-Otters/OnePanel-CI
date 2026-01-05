<script lang="ts">
    import { onMount } from "svelte";
    import * as Table from "$lib/components/ui/table";
    import { Button } from "$lib/components/ui/button";
    import { Loader2, RefreshCw } from "lucide-svelte";
    import { listComposes } from "$lib/api";
    import { t } from "svelte-i18n";
    import { toast } from "svelte-sonner";
    import clsx from "clsx";

    import ComposeContentDialog from "./ComposeContentDialog.svelte";

    let { serverId, loading = $bindable(false) } = $props<{
        serverId: number;
        loading?: boolean;
    }>();

    let composes = $state<any[]>([]);
    let viewDialogOpen = $state(false);
    let viewPath = $state("");
    let viewName = $state("");

    async function refresh() {
        loading = true;
        try {
            composes = await listComposes(serverId);
        } catch (e: any) {
            console.error(e);
            toast.error(
                $t("servers.compose_list.error_loading", { error: e.message }),
            );
        } finally {
            loading = false;
        }
    }

    function openViewDialog(item: any) {
        viewPath = item.path;
        viewName = item.name;
        viewDialogOpen = true;
    }

    onMount(() => {
        refresh();
    });

    export { refresh };
</script>

<div class="space-y-4">
    <div class="rounded-md border">
        <Table.Root>
            <Table.Header>
                <Table.Row>
                    <Table.Head
                        >{$t("servers.compose_list.table.name")}</Table.Head
                    >
                    <Table.Head
                        >{$t("servers.compose_list.table.status")}</Table.Head
                    >
                    <Table.Head
                        >{$t("servers.compose_list.table.path")}</Table.Head
                    >
                    <Table.Head
                        >{$t("servers.compose_list.table.created")}</Table.Head
                    >
                    <Table.Head class="text-right"
                        >{$t("common.actions")}</Table.Head
                    >
                </Table.Row>
            </Table.Header>
            <Table.Body>
                {#if loading && composes.length === 0}
                    <Table.Row>
                        <Table.Cell
                            colspan={5}
                            class="text-center py-8 text-muted-foreground"
                        >
                            <Loader2
                                class="h-6 w-6 animate-spin mx-auto mb-2"
                            />
                            {$t("servers.compose_list.loading")}
                        </Table.Cell>
                    </Table.Row>
                {:else if composes.length === 0}
                    <Table.Row>
                        <Table.Cell
                            colspan={5}
                            class="text-center py-8 text-muted-foreground"
                        >
                            {$t("servers.compose_list.no_composes")}
                        </Table.Cell>
                    </Table.Row>
                {:else}
                    {#each composes as item}
                        <Table.Row>
                            <Table.Cell class="font-medium"
                                >{item.name}</Table.Cell
                            >
                            <Table.Cell>
                                {#if item.containers && item.containers.length > 0}
                                    <div class="flex flex-col gap-1">
                                        {#each item.containers as c}
                                            <span
                                                class={clsx(
                                                    "inline-flex items-center px-2 py-0.5 rounded text-xs font-medium",
                                                    c.state === "running"
                                                        ? "bg-green-100 text-green-800"
                                                        : "bg-yellow-100 text-yellow-800",
                                                )}
                                            >
                                                {c.name}: {c.state}
                                            </span>
                                        {/each}
                                    </div>
                                {:else}
                                    -
                                {/if}
                            </Table.Cell>
                            <Table.Cell class="font-mono text-xs"
                                >{item.path || "-"}</Table.Cell
                            >
                            <Table.Cell>{item.createdAt || "-"}</Table.Cell>
                            <Table.Cell class="text-right">
                                <Button
                                    variant="outline"
                                    size="sm"
                                    onclick={() => openViewDialog(item)}
                                >
                                    {$t("common.edit")}
                                </Button>
                            </Table.Cell>
                        </Table.Row>
                    {/each}
                {/if}
            </Table.Body>
        </Table.Root>
    </div>

    <ComposeContentDialog
        bind:open={viewDialogOpen}
        {serverId}
        path={viewPath}
        name={viewName}
    />
</div>
