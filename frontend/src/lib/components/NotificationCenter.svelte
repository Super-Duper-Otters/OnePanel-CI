<script lang="ts">
    import { Bell, CheckCircle2, AlertCircle, Trash2, X } from "lucide-svelte";
    import { Button } from "$lib/components/ui/button";
    import * as Popover from "$lib/components/ui/popover";
    import * as ScrollArea from "$lib/components/ui/scroll-area";
    import { notificationStore } from "$lib/stores/notifications.svelte";
    import { t } from "svelte-i18n";
    import { fade, fly } from "svelte/transition";

    function formatTime(timestamp: number) {
        return new Date(timestamp).toLocaleTimeString();
    }

    function formatDuration(ms?: number) {
        if (!ms) return "";
        const seconds = Math.floor(ms / 1000);
        if (seconds < 60) return `${seconds}s`;
        const minutes = Math.floor(seconds / 60);
        const remainingSeconds = seconds % 60;
        return `${minutes}m ${remainingSeconds}s`;
    }
</script>

<Popover.Root>
    <Popover.Trigger>
        <Button variant="ghost" size="icon" class="relative">
            <Bell class="h-5 w-5" />
            {#if notificationStore.all.length > 0}
                <span
                    class="absolute top-1.5 right-1.5 h-2 w-2 rounded-full bg-red-500"
                ></span>
            {/if}
        </Button>
    </Popover.Trigger>
    <Popover.Content class="w-80 p-0" align="end">
        <div class="flex items-center justify-between p-4 border-b">
            <h4 class="font-medium leading-none">
                {$t("notifications.title") || "Notifications"}
            </h4>
            {#if notificationStore.all.length > 0}
                <Button
                    variant="ghost"
                    size="icon"
                    class="h-6 w-6"
                    onclick={() => notificationStore.clear()}
                    title={$t("notifications.clear") || "Clear all"}
                >
                    <Trash2 class="h-4 w-4 text-muted-foreground" />
                </Button>
            {/if}
        </div>
        <ScrollArea.Root class="h-[300px]">
            <div class="p-4 gap-4 flex flex-col">
                {#if notificationStore.all.length === 0}
                    <div
                        class="flex flex-col items-center justify-center py-8 text-center text-muted-foreground"
                    >
                        <Bell class="h-8 w-8 mb-2 opacity-50" />
                        <p class="text-sm">
                            {$t("notifications.empty") || "No notifications"}
                        </p>
                    </div>
                {:else}
                    {#each notificationStore.all as item (item.id)}
                        <div
                            class="flex gap-3 items-start p-3 rounded-lg border bg-card text-card-foreground shadow-sm"
                            in:fly={{ y: -10, duration: 200 }}
                        >
                            {#if item.status === "success"}
                                <CheckCircle2
                                    class="h-5 w-5 text-green-500 mt-0.5"
                                />
                            {:else}
                                <AlertCircle
                                    class="h-5 w-5 text-red-500 mt-0.5"
                                />
                            {/if}
                            <div class="flex-1 space-y-1">
                                <p class="text-sm font-medium leading-none">
                                    {item.title}
                                </p>
                                <p
                                    class="text-xs text-muted-foreground break-all"
                                >
                                    {item.detail}
                                </p>
                                <div
                                    class="flex items-center gap-2 text-[10px] text-muted-foreground mt-1"
                                >
                                    <span>{formatTime(item.timestamp)}</span>
                                    {#if item.duration}
                                        <span>•</span>
                                        <span
                                            >{formatDuration(
                                                item.duration,
                                            )}</span
                                        >
                                    {/if}
                                </div>
                            </div>
                        </div>
                    {/each}
                {/if}
            </div>
        </ScrollArea.Root>
    </Popover.Content>
</Popover.Root>
