<script lang="ts">
    import { Popover as PopoverPrimitive } from "bits-ui";

    let { children, ...restProps } = $props();
</script>

<PopoverPrimitive.Root {...restProps}>
    {@render children?.()}
</PopoverPrimitive.Root>
