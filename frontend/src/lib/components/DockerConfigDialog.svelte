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
    import { t } from "svelte-i18n";
    import { toast } from "svelte-sonner";

    let { open = $bindable(false), imageName = $bindable("") } = $props<{
        open?: boolean;
        imageName: string;
    }>();

    let localName = $state(imageName);

    $effect(() => {
        if (open) {
            localName = imageName;
        }
    });

    function save() {
        if (!localName) {
            toast.error($t("docker.config.name_required"));
            return;
        }
        imageName = localName;
        open = false;
        toast.success($t("docker.config.saved"));
    }
</script>

<Dialog bind:open>
    <DialogContent class="sm:max-w-[425px]">
        <DialogHeader>
            <DialogTitle>{$t("docker.config.title")}</DialogTitle>
        </DialogHeader>
        <div class="grid gap-4 py-4">
            <div class="grid grid-cols-4 items-center gap-4">
                <span class="text-right"
                    >{$t("docker.config.image_name_label")}</span
                >
                <Input
                    bind:value={localName}
                    placeholder={$t("docker.config.placeholder")}
                    class="col-span-3"
                />
            </div>
        </div>
        <DialogFooter>
            <Button onclick={save}>{$t("settings.save") || "Save"}</Button>
        </DialogFooter>
    </DialogContent>
</Dialog>
