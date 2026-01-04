<script lang="ts">
    import { onMount } from "svelte";
    import {
        Folder,
        FolderOpen,
        File,
        ChevronRight,
        Check,
    } from "lucide-svelte";
    import { Button } from "$lib/components/ui/button";
    import { ScrollArea } from "$lib/components/ui/scroll-area";
    import { t } from "svelte-i18n";

    interface FileEntry {
        name: string;
        path: string;
        is_dir: boolean;
    }

    let { onselect, oncancel } = $props<{
        onselect: (path: string) => void;
        oncancel: () => void;
    }>();

    let currentPath = $state<string | null>(null);
    let entries = $state<FileEntry[]>([]);
    let loading = $state(false);

    async function loadDirectory(path: string | null = null) {
        loading = true;
        try {
            const res = await fetch("http://localhost:3000/api/fs/list", {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify({ path }),
            });
            if (res.ok) {
                entries = await res.json();
                if (path) currentPath = path;
                else if (entries.length > 0 && entries[0].name !== "..") {
                }
            }
        } catch (e) {
            console.error(e);
        } finally {
            loading = false;
        }
    }

    async function handleEntryClick(entry: FileEntry) {
        if (entry.is_dir) {
            await loadDirectory(entry.path);
        }
    }

    function handleSelectCurrent() {
        if (currentPath) {
            onselect(currentPath);
        }
    }

    onMount(() => {
        loadDirectory(null);
    });
</script>

<div class="border rounded-md p-4 bg-background">
    <div class="flex items-center justify-between mb-4">
        <h3 class="text-lg font-semibold flex items-center gap-2">
            <FolderOpen size={20} />
            {$t("picker.title")}
        </h3>
    </div>

    <div
        class="mb-2 text-sm text-muted-foreground break-all bg-muted p-2 rounded"
    >
        {currentPath || $t("picker.current")}
    </div>

    <ScrollArea class="h-[300px] border rounded-md p-2">
        {#if loading}
            <div class="flex justify-center p-4">{$t("picker.loading")}</div>
        {:else}
            <ul class="space-y-1">
                {#each entries as entry}
                    <!-- svelte-ignore a11y_click_events_have_key_events -->
                    <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
                    <li
                        class="flex items-center gap-2 p-2 hover:bg-accent hover:text-accent-foreground rounded cursor-pointer"
                        onclick={() => handleEntryClick(entry)}
                    >
                        {#if entry.name === ".."}
                            <ChevronRight class="rotate-180" size={16} />
                            <span class="font-medium">..</span>
                        {:else if entry.is_dir}
                            <Folder size={16} class="text-blue-500" />
                            <span>{entry.name}</span>
                        {:else}
                            <File size={16} class="text-gray-400" />
                            <span class="text-muted-foreground"
                                >{entry.name}</span
                            >
                        {/if}
                    </li>
                {/each}
            </ul>
        {/if}
    </ScrollArea>

    <div class="flex justify-end gap-2 mt-4">
        <Button variant="outline" onclick={oncancel}
            >{$t("picker.cancel")}</Button
        >
        <Button onclick={handleSelectCurrent} disabled={!currentPath}>
            <Check size={16} class="mr-2" />
            {$t("picker.select")}
        </Button>
    </div>
</div>
