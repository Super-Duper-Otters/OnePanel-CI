<script lang="ts">
    import { onMount } from "svelte";
    import { t } from "svelte-i18n";
    import {
        Table,
        TableBody,
        TableCell,
        TableHead,
        TableHeader,
        TableRow,
    } from "$lib/components/ui/table";
    import { Button } from "$lib/components/ui/button";
    import { Input } from "$lib/components/ui/input";
    import { Loader2, Trash2, RefreshCw } from "lucide-svelte";
    import * as Dialog from "$lib/components/ui/dialog";

    interface DockerImage {
        id: string;
        tags: string[];
        created: number;
        size: number;
    }

    let images = $state<DockerImage[]>([]);
    let loading = $state(false);
    let operationLoading = $state<string | null>(null);
    let pullOpen = $state(false);
    let pullImageName = $state("");
    let pullLoading = $state(false);

    async function fetchImages() {
        loading = true;
        try {
            const res = await fetch("http://localhost:3000/api/docker/images");
            if (res.ok) {
                images = await res.json();
            }
        } catch (error) {
            console.error(error);
        } finally {
            loading = false;
        }
    }

    async function removeImage(id: string) {
        if (
            !confirm(
                $t("docker.images_panel.confirm_remove") ||
                    "Are you sure you want to remove this image?",
            )
        )
            return;
        operationLoading = id;
        try {
            const res = await fetch(
                `http://localhost:3000/api/docker/images/${id}`,
                {
                    method: "DELETE",
                },
            );
            if (res.ok) {
                await fetchImages();
            } else {
                alert(
                    $t("docker.images_panel.remove_failed") ||
                        "Failed to remove image",
                );
            }
        } catch (error) {
            console.error(error);
        } finally {
            operationLoading = null;
        }
    }

    async function pullImage() {
        if (!pullImageName) return;
        pullLoading = true;
        try {
            const res = await fetch(
                "http://localhost:3000/api/docker/images/pull",
                {
                    method: "POST",
                    headers: { "Content-Type": "application/json" },
                    body: JSON.stringify({ image: pullImageName }),
                },
            );
            if (res.ok) {
                pullOpen = false;
                pullImageName = "";
                await fetchImages();
            } else {
                alert(
                    $t("docker.images_panel.pull_failed") ||
                        "Failed to pull image",
                );
            }
        } catch (error) {
            console.error(error);
        } finally {
            pullLoading = false;
        }
    }

    function formatSize(bytes: number) {
        if (bytes === 0) return "0 B";
        const k = 1024;
        const sizes = ["B", "KB", "MB", "GB", "TB"];
        const i = Math.floor(Math.log(bytes) / Math.log(k));
        return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + " " + sizes[i];
    }

    function formatDate(timestamp: number) {
        return new Date(timestamp * 1000).toLocaleString();
    }

    export function openPullDialog() {
        pullOpen = true;
    }

    onMount(() => {
        fetchImages();
    });
</script>

<div class="space-y-4">
    <div class="rounded-md border">
        <Table>
            <TableHeader>
                <TableRow>
                    <TableHead
                        >{$t("docker.images_panel.table.id") || "ID"}</TableHead
                    >
                    <TableHead
                        >{$t("docker.images_panel.table.tag") ||
                            "Tags"}</TableHead
                    >
                    <TableHead
                        >{$t("docker.images_panel.table.size") ||
                            "Size"}</TableHead
                    >
                    <TableHead
                        >{$t("docker.images_panel.table.created") ||
                            "Created"}</TableHead
                    >
                    <TableHead class="text-right"
                        >{$t("common.actions") || "Actions"}</TableHead
                    >
                </TableRow>
            </TableHeader>
            <TableBody>
                {#if loading && images.length === 0}
                    <TableRow>
                        <TableCell colspan={5} class="text-center h-24">
                            <Loader2 class="animate-spin mx-auto" />
                        </TableCell>
                    </TableRow>
                {:else if images.length === 0}
                    <TableRow>
                        <TableCell colspan={5} class="text-center h-24">
                            {$t(
                                "docker.images_panel.no_images_found_generic",
                            ) || "No images found"}
                        </TableCell>
                    </TableRow>
                {:else}
                    {#each images as image}
                        <TableRow>
                            <TableCell class="font-mono text-xs"
                                >{image.id}</TableCell
                            >
                            <TableCell>
                                <div class="flex flex-col gap-1">
                                    {#each image.tags as tag}
                                        <span class="text-xs">{tag}</span>
                                    {/each}
                                </div>
                            </TableCell>
                            <TableCell>{formatSize(image.size)}</TableCell>
                            <TableCell>{formatDate(image.created)}</TableCell>
                            <TableCell class="text-right">
                                <Button
                                    variant="ghost"
                                    size="icon"
                                    class="text-red-500 hover:text-red-700 hover:bg-red-50"
                                    disabled={operationLoading === image.id}
                                    onclick={() => removeImage(image.id)}
                                >
                                    {#if operationLoading === image.id}
                                        <Loader2
                                            class="animate-spin"
                                            size={16}
                                        />
                                    {:else}
                                        <Trash2 size={16} />
                                    {/if}
                                </Button>
                            </TableCell>
                        </TableRow>
                    {/each}
                {/if}
            </TableBody>
        </Table>
    </div>

    <Dialog.Root bind:open={pullOpen}>
        <Dialog.Content>
            <Dialog.Header>
                <Dialog.Title
                    >{$t("docker.images_panel.pull_image") ||
                        "Pull Image"}</Dialog.Title
                >
            </Dialog.Header>
            <div class="py-4 space-y-4">
                <div class="space-y-2">
                    <label for="image-name" class="text-sm font-medium"
                        >{$t("docker.images_panel.image_name") ||
                            "Image Name"}</label
                    >
                    <Input
                        id="image-name"
                        placeholder="e.g. nginx:latest"
                        bind:value={pullImageName}
                        disabled={pullLoading}
                    />
                </div>
            </div>
            <Dialog.Footer>
                <Button
                    variant="outline"
                    onclick={() => (pullOpen = false)}
                    disabled={pullLoading}
                >
                    {$t("picker.cancel") || "Cancel"}
                </Button>
                <Button
                    onclick={pullImage}
                    disabled={pullLoading || !pullImageName}
                >
                    {#if pullLoading}
                        <Loader2 class="animate-spin mr-2" size={16} />
                    {/if}
                    {$t("docker.images_panel.pull") || "Pull"}
                </Button>
            </Dialog.Footer>
        </Dialog.Content>
    </Dialog.Root>
</div>
