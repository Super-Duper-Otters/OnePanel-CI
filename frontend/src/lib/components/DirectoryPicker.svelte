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
    import { cn } from "$lib/utils";

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
                // If path is null (root), update currentPath from the first entry if available?
                // Actually, listing "/" or "." usually returns absolute paths.
                // We'll rely on the server to handle "null" as cwd.
                // But for display, if we navigated, use that.
                if (path) currentPath = path;
                else if (entries.length > 0 && entries[0].name !== "..") {
                    // Heuristic: if listing root, assume cwd is parent of items?
                    // Or just trust the server returned proper paths.
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
            Browse Directory
        </h3>
    </div>

    <div
        class="mb-2 text-sm text-muted-foreground break-all bg-muted p-2 rounded"
    >
        {currentPath || "Current Directory"}
    </div>

    <ScrollArea class="h-[300px] border rounded-md p-2">
        {#if loading}
            <div class="flex justify-center p-4">Loading...</div>
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
        <Button variant="outline" onclick={oncancel}>Cancel</Button>
        <Button onclick={handleSelectCurrent} disabled={!currentPath}>
            <Check size={16} class="mr-2" />
            Select This Folder
        </Button>
    </div>
</div>
