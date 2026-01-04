<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import {
    Card,
    CardContent,
    CardHeader,
    CardTitle,
  } from "$lib/components/ui/card";
  let { onadded } = $props<{ onadded?: () => void }>();
  let path = $state("");

  async function addDirectory() {
    if (!path) return;

    try {
      const response = await fetch("http://localhost:3000/api/directories", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ path }),
      });

      if (response.ok) {
        path = "";
        onadded?.(); // Notify parent to refresh
      } else {
        alert("Failed to add directory");
      }
    } catch (e) {
      console.error(e);
      alert("Error adding directory");
    }
  }
</script>

<Card class="mb-4">
  <CardHeader>
    <CardTitle>Add Repository</CardTitle>
  </CardHeader>
  <CardContent class="flex gap-4">
    <Input
      type="text"
      placeholder="Absolute path to git repository"
      bind:value={path}
    />
    <Button onclick={addDirectory}>Add</Button>
  </CardContent>
</Card>
