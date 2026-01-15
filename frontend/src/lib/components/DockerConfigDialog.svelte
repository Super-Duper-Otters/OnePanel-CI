<script lang="ts">
    import { Button } from "$lib/components/ui/button";
    import { Input } from "$lib/components/ui/input";
    import {
        Dialog,
        DialogContent,
        DialogHeader,
        DialogTitle,
        DialogFooter,
    } from "$lib/components/ui/dialog";
    import * as Select from "$lib/components/ui/select";
    import { Label } from "$lib/components/ui/label";
    import { t } from "svelte-i18n";
    import { toast } from "svelte-sonner";
    import { listServers, listComposes } from "$lib/api";

    let {
        open = $bindable(false),
        imageName = $bindable(""),
        defaultServerId = $bindable(undefined),
        defaultComposePath = $bindable(undefined),
        onSave,
    } = $props<{
        open?: boolean;
        imageName: string;
        defaultServerId?: number;
        defaultComposePath?: string;
        onSave?: (
            name: string,
            serverId?: number,
            composePath?: string,
        ) => void;
    }>();

    let localName = $state(imageName);
    let localServerId = $state<string | undefined>(defaultServerId?.toString());
    let localComposePath = $state<string | undefined>(defaultComposePath);

    let servers = $state<any[]>([]);
    let composes = $state<any[]>([]);
    let loadingComposes = $state(false);

    $effect(() => {
        if (open) {
            localName = imageName;
            localServerId = defaultServerId?.toString();
            localComposePath = defaultComposePath;
            loadServers();
        }
    });

    $effect(() => {
        if (localServerId) {
            loadComposes(parseInt(localServerId));
        } else {
            composes = [];
        }
    });

    async function loadServers() {
        try {
            const res = await listServers();
            servers = res.map((s: any) => ({
                value: s.id.toString(),
                label: `${s.name} (${s.host})`,
            }));
        } catch (e) {
            console.error(e);
            toast.error("Failed to load servers");
        }
    }

    async function loadComposes(serverId: number) {
        loadingComposes = true;
        try {
            const res = await listComposes(serverId);
            composes = res.map((c: any) => ({ value: c.path, label: c.name }));
        } catch (e) {
            console.error(e);
            toast.error("Failed to load composes");
        } finally {
            loadingComposes = false;
        }
    }

    function save() {
        if (!localName) {
            toast.error($t("docker.config.name_required"));
            return;
        }

        const sId = localServerId ? parseInt(localServerId) : undefined;

        if (onSave) {
            onSave(localName, sId, localComposePath);
        } else {
            imageName = localName;
            defaultServerId = sId;
            defaultComposePath = localComposePath;
        }

        open = false;
        // Success toast handled by parent or here if simple update
        // toast.success($t("docker.config.saved"));
    }
</script>

<Dialog bind:open>
    <DialogContent class="sm:max-w-[500px]">
        <DialogHeader>
            <DialogTitle>{$t("docker.config.title")}</DialogTitle>
        </DialogHeader>
        <div class="grid gap-6 py-4">
            <div class="space-y-2">
                <Label>{$t("docker.config.image_name_label")}</Label>
                <Input
                    bind:value={localName}
                    placeholder={$t("docker.config.placeholder")}
                />
            </div>

            <div class="space-y-4 border-t pt-4">
                <h4 class="font-medium">{$t("docker.config.bind_target")}</h4>
                <div class="space-y-2">
                    <Label>{$t("docker.config.default_server")}</Label>
                    <Select.Root type="single" bind:value={localServerId}>
                        <Select.Trigger>
                            {servers.find((s) => s.value === localServerId)
                                ?.label || $t("docker.config.select_server")}
                        </Select.Trigger>
                        <Select.Content>
                            {#each servers as s}
                                <Select.Item value={s.value} label={s.label}
                                    >{s.label}</Select.Item
                                >
                            {/each}
                        </Select.Content>
                    </Select.Root>
                </div>

                <div class="space-y-2">
                    <Label>{$t("docker.config.default_compose")}</Label>
                    <Select.Root
                        type="single"
                        bind:value={localComposePath}
                        disabled={!localServerId}
                    >
                        <Select.Trigger>
                            {composes.find((c) => c.value === localComposePath)
                                ?.label || $t("docker.config.select_compose")}
                        </Select.Trigger>
                        <Select.Content>
                            {#each composes as c}
                                <Select.Item value={c.value} label={c.label}
                                    >{c.label}</Select.Item
                                >
                            {/each}
                        </Select.Content>
                    </Select.Root>
                </div>
            </div>
        </div>
        <DialogFooter>
            <Button onclick={save}>{$t("settings.save") || "Save"}</Button>
        </DialogFooter>
    </DialogContent>
</Dialog>
