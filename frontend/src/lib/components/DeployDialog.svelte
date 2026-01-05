<script lang="ts">
    import {
        Dialog,
        DialogContent,
        DialogHeader,
        DialogTitle,
        DialogDescription,
    } from "$lib/components/ui/dialog";
    import { Button } from "$lib/components/ui/button";
    import { Input } from "$lib/components/ui/input";
    import { Label } from "$lib/components/ui/label";
    import { listServers } from "$lib/api"; // This import is kept, but the implementation of loadServers changes.
    import { Loader2, ServerIcon, Upload } from "lucide-svelte"; // Added ServerIcon, Upload
    import { t } from "svelte-i18n";
    import { toast } from "svelte-sonner";
    import clsx from "clsx"; // Added clsx import

    let {
        open = $bindable(false),
        imageTag,
        onpush,
    } = $props<{
        open: boolean;
        imageTag: string;
        onpush: (serverId: number) => void;
    }>();

    let servers = $state<any[]>([]);
    let loading = $state(false);
    let selectedId = $state<number | null>(null);

    $effect(() => {
        if (open) {
            loadServers();
        }
    });

    async function loadServers() {
        loading = true;
        try {
            const res = await fetch("http://localhost:3000/api/servers");
            if (res.ok) {
                servers = await res.json();
            }
        } catch (e) {
            console.error(e);
            toast.error("Failed to load servers");
        } finally {
            loading = false;
        }
    }

    function handlePush() {
        if (selectedId !== null) {
            onpush(selectedId);
            open = false;
        }
    }
</script>

<Dialog bind:open>
    <DialogContent class="max-w-xl">
        <DialogHeader>
            <DialogTitle>Push Image to Server</DialogTitle>
            <DialogDescription
                >Select a server to push the image <b>{imageTag}</b
                >.</DialogDescription
            >
        </DialogHeader>

        <div class="space-y-4 py-4 max-h-[60vh] overflow-y-auto">
            <!-- Server Selection -->
            <div class="space-y-2">
                <Label>Target Server</Label>
                {#if loading}
                    <div
                        class="flex items-center gap-2 text-muted-foreground p-2"
                    >
                        <Loader2 class="h-4 w-4 animate-spin" />
                        <span>Loading servers...</span>
                    </div>
                {:else if servers.length === 0}
                    <div class="text-muted-foreground p-2">
                        No servers found.
                    </div>
                {:else}
                    <div class="grid gap-2">
                        {#each servers as server}
                            <button
                                class={clsx(
                                    "flex items-center gap-3 p-3 rounded-lg border transition-all hover:bg-muted",
                                    selectedId === server.id
                                        ? "border-primary bg-primary/5 ring-1 ring-primary"
                                        : "border-border",
                                )}
                                onclick={() => (selectedId = server.id)}
                            >
                                <div class="bg-primary/10 p-2 rounded-md">
                                    <ServerIcon class="h-5 w-5 text-primary" />
                                </div>
                                <div class="flex flex-col items-start">
                                    <span class="font-medium text-sm"
                                        >{server.name}</span
                                    >
                                    <span class="text-xs text-muted-foreground"
                                        >{server.host}:{server.port}</span
                                    >
                                </div>
                            </button>
                        {/each}
                    </div>
                {/if}
            </div>
        </div>

        <div class="flex justify-end gap-2">
            <Button variant="ghost" onclick={() => (open = false)}
                >{$t("common.cancel", { default: "Cancel" })}</Button
            >
            <Button onclick={handlePush} disabled={selectedId === null}>
                <Upload class="mr-2 h-4 w-4" />
                Push Image
            </Button>
        </div>
    </DialogContent>
</Dialog>
