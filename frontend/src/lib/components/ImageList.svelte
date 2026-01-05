<script lang="ts">
    import { onMount } from "svelte";
    import * as Table from "$lib/components/ui/table";
    import { Button } from "$lib/components/ui/button";
    import { Loader2, Trash2, RefreshCw } from "lucide-svelte";
    import { listImages, removeImage } from "$lib/api";
    import { t } from "svelte-i18n";
    import { toast } from "svelte-sonner";

    let { serverId } = $props<{ serverId: number }>();

    let images = $state<any[]>([]);
    let loading = $state(false);
    let removing = $state<string | null>(null);

    async function refresh() {
        loading = true;
        try {
            images = await listImages(serverId);
        } catch (e: any) {
            console.error(e);
            toast.error(
                $t("servers.images.error_loading", { error: e.message }),
            );
        } finally {
            loading = false;
        }
    }

    async function handleRemove(id: string) {
        if (!confirm($t("servers.images.confirm_remove"))) return;
        removing = id;
        try {
            await removeImage(serverId, id, true);
            toast.success($t("servers.images.remove_success"));
            await refresh();
        } catch (e: any) {
            console.error(e);
            toast.error($t("servers.images.remove_error") + ": " + e.message);
        } finally {
            removing = null;
        }
    }

    onMount(() => {
        refresh();
    });

    // Helper to format size
    function formatSize(bytes: number) {
        if (bytes === 0) return "0 B";
        const k = 1024;
        const sizes = ["B", "KB", "MB", "GB", "TB"];
        const i = Math.floor(Math.log(bytes) / Math.log(k));
        return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + " " + sizes[i];
    }
</script>

<div class="space-y-4">
    <div class="rounded-md border">
        <Table.Root>
            <Table.Header>
                <Table.Row>
                    <Table.Head>{$t("servers.images.table.tags")}</Table.Head>
                    <Table.Head class="w-[100px]"
                        >{$t("servers.images.table.id")}</Table.Head
                    >
                    <Table.Head>{$t("servers.images.table.size")}</Table.Head>
                    <Table.Head>{$t("servers.images.table.created")}</Table.Head
                    >
                    <Table.Head>{$t("servers.images.table.status")}</Table.Head>
                    <Table.Head class="text-right"
                        >{$t("common.actions")}</Table.Head
                    >
                </Table.Row>
            </Table.Header>
            <Table.Body>
                {#if loading && images.length === 0}
                    <Table.Row>
                        <Table.Cell
                            colspan={6}
                            class="text-center py-8 text-muted-foreground"
                        >
                            <Loader2
                                class="h-6 w-6 animate-spin mx-auto mb-2"
                            />
                            {$t("servers.images.loading")}
                        </Table.Cell>
                    </Table.Row>
                {:else if images.length === 0}
                    <Table.Row>
                        <Table.Cell
                            colspan={6}
                            class="text-center py-8 text-muted-foreground"
                        >
                            {$t("servers.images.no_images")}
                        </Table.Cell>
                    </Table.Row>
                {:else}
                    {#each images as item}
                        <Table.Row>
                            <Table.Cell>
                                {#if item.tags && item.tags.length > 0}
                                    <div class="flex flex-col gap-1">
                                        {#each item.tags as tag}
                                            <span
                                                class="inline-flex w-fit items-center px-2 py-0.5 rounded text-xs font-medium bg-blue-100 text-blue-800"
                                            >
                                                {tag}
                                            </span>
                                        {/each}
                                    </div>
                                {:else}
                                    <span class="text-muted-foreground text-xs"
                                        >&lt;none&gt;</span
                                    >
                                {/if}
                            </Table.Cell>
                            <Table.Cell class="font-mono text-xs"
                                >{(item.id || "").substring(7, 19)}</Table.Cell
                            >
                            <Table.Cell>{item.size}</Table.Cell>
                            <Table.Cell
                                >{new Date(
                                    item.createdAt,
                                ).toLocaleString()}</Table.Cell
                            >
                            <Table.Cell>
                                {#if item.isUsed}
                                    <span
                                        class="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-green-100 text-green-800"
                                    >
                                        {$t("servers.images.status.used")}
                                    </span>
                                {:else}
                                    <span
                                        class="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-gray-100 text-gray-800"
                                    >
                                        {$t("servers.images.status.unused")}
                                    </span>
                                {/if}
                            </Table.Cell>
                            <Table.Cell class="text-right">
                                <Button
                                    variant="ghost"
                                    size="icon"
                                    onclick={() => handleRemove(item.id)}
                                    disabled={removing === item.id}
                                    class="text-destructive hover:text-destructive"
                                >
                                    {#if removing === item.id}
                                        <Loader2 class="h-4 w-4 animate-spin" />
                                    {:else}
                                        <Trash2 class="h-4 w-4" />
                                    {/if}
                                </Button>
                            </Table.Cell>
                        </Table.Row>
                    {/each}
                {/if}
            </Table.Body>
        </Table.Root>
    </div>
</div>
