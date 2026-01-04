<script lang="ts">
    import { Button } from "$lib/components/ui/button";
    import { Input } from "$lib/components/ui/input";
    import {
        Dialog,
        DialogContent,
        DialogHeader,
        DialogTitle,
    } from "$lib/components/ui/dialog";
    import { t } from "svelte-i18n";
    import { toast } from "svelte-sonner";

    let { open = $bindable(false), onadded } = $props<{
        open?: boolean;
        onadded?: () => void;
    }>();

    let host = $state("");
    let port = $state(10080);
    let apiKey = $state("");
    let name = $state("");
    let testing = $state(false);

    async function handleAdd() {
        if (!host || !apiKey || !name) {
            toast.error("Please fill all fields");
            return;
        }

        try {
            const res = await fetch("http://localhost:3000/api/servers", {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify({
                    name,
                    host,
                    port,
                    api_key: apiKey,
                }),
            });

            if (res.ok) {
                toast.success($t("settings.save_success"));
                onadded?.();
                open = false;
                // Reset
                host = "";
                port = 10080;
                apiKey = "";
                name = "";
            } else {
                toast.error($t("settings.save_error"));
            }
        } catch (e) {
            console.error(e);
            toast.error($t("settings.save_error"));
        }
    }
</script>

<Dialog bind:open>
    <DialogContent class="sm:max-w-[425px]">
        <DialogHeader>
            <DialogTitle>{$t("servers.add_server_title")}</DialogTitle>
        </DialogHeader>
        <div class="grid gap-4 py-4">
            <div class="grid grid-cols-4 items-center gap-4">
                <span class="text-right">{$t("servers.name")}</span>
                <Input bind:value={name} class="col-span-3" />
            </div>
            <div class="grid grid-cols-4 items-center gap-4">
                <span class="text-right">{$t("settings.host")}</span>
                <Input
                    bind:value={host}
                    placeholder="e.g. 192.168.1.100"
                    class="col-span-3"
                />
            </div>
            <div class="grid grid-cols-4 items-center gap-4">
                <span class="text-right">{$t("settings.port")}</span>
                <Input type="number" bind:value={port} class="col-span-3" />
            </div>
            <div class="grid grid-cols-4 items-center gap-4">
                <span class="text-right">{$t("settings.api_key")}</span>
                <Input type="password" bind:value={apiKey} class="col-span-3" />
            </div>
        </div>
        <div class="flex justify-end gap-2">
            <Button onclick={handleAdd}>{$t("directory.add_button")}</Button>
        </div>
    </DialogContent>
</Dialog>
