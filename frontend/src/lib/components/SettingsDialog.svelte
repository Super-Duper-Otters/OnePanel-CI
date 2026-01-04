<script lang="ts">
    import { Button } from "$lib/components/ui/button";
    import {
        Dialog,
        DialogContent,
        DialogHeader,
        DialogTitle,
    } from "$lib/components/ui/dialog";
    import { t, locale } from "svelte-i18n";

    let { open = $bindable(false) } = $props<{ open?: boolean }>();

    function toggleLanguage() {
        locale.update((l) => (l === "zh-CN" ? "en" : "zh-CN"));
    }
</script>

<Dialog bind:open>
    <DialogContent class="sm:max-w-[425px]">
        <DialogHeader>
            <DialogTitle>{$t("settings.title")}</DialogTitle>
        </DialogHeader>
        <div class="grid gap-4 py-4">
            <div class="flex items-center justify-between">
                <span>Language / 语言</span>
                <Button variant="outline" onclick={toggleLanguage}>
                    {$locale === "zh-CN" ? "English" : "中文"}
                </Button>
            </div>
            <!-- Future settings can go here -->
        </div>
    </DialogContent>
</Dialog>
