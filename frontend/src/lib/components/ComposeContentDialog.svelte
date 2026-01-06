<script lang="ts">
    import * as Dialog from "$lib/components/ui/dialog";
    import { Button } from "$lib/components/ui/button";
    import { Loader2 } from "lucide-svelte";
    import { getComposeContent, updateComposeContent } from "$lib/api";
    import { t } from "svelte-i18n";
    import { toast } from "svelte-sonner";

    let {
        open = $bindable(false),
        serverId,
        path,
        name,
    }: {
        open: boolean;
        serverId: number;
        path: string;
        name: string;
    } = $props();

    let content = $state("");
    let loading = $state(false);
    let saving = $state(false);

    $effect(() => {
        if (open && path) {
            loadContent();
        } else {
            content = "";
        }
    });

    async function loadContent() {
        loading = true;
        try {
            content = await getComposeContent(serverId, path);
        } catch (e: any) {
            console.error(e);
            toast.error(
                $t("servers.compose_content.error_loading", {
                    error: e.message,
                }),
            );
            open = false;
        } finally {
            loading = false;
        }
    }

    async function saveContent() {
        saving = true;
        try {
            await updateComposeContent(serverId, name, path, content);
            toast.success($t("settings.save_success"));
            open = false;
        } catch (e: any) {
            console.error(e);
            toast.error($t("settings.save_error") + ": " + e.message);
        } finally {
            saving = false;
        }
    }
</script>

<Dialog.Root bind:open>
    <Dialog.Content class="max-w-4xl max-h-[80vh] flex flex-col">
        <Dialog.Header>
            <Dialog.Title
                >{$t("servers.compose_content.title")}: {name}</Dialog.Title
            >
            <Dialog.Description
                class="font-mono text-xs text-muted-foreground break-all"
            >
                {path}
            </Dialog.Description>
        </Dialog.Header>

        <div class="flex-1 overflow-hidden p-1">
            {#if loading}
                <div class="flex items-center justify-center h-full">
                    <Loader2 class="h-6 w-6 animate-spin mr-2" />
                    <span>{$t("servers.compose_content.loading")}</span>
                </div>
            {:else}
                <textarea
                    class="w-full h-full min-h-[400px] bg-muted p-4 rounded-md font-mono text-sm resize-none focus:outline-none focus:ring-2 focus:ring-ring"
                    bind:value={content}
                    spellcheck="false"
                ></textarea>
            {/if}
        </div>

        <Dialog.Footer>
            <Button variant="outline" onclick={() => (open = false)}
                >{$t("common.close")}</Button
            >
            <Button onclick={saveContent} disabled={loading || saving}>
                {#if saving}
                    <Loader2 class="h-4 w-4 animate-spin mr-2" />
                {/if}
                {$t("settings.save")}
            </Button>
        </Dialog.Footer>
    </Dialog.Content>
</Dialog.Root>
