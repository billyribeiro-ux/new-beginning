<script lang="ts">
	import Seo from '$lib/components/seo/Seo.svelte';
	import AuthShell from '$lib/components/layout/AuthShell.svelte';
	import AuthForm from '$lib/components/forms/AuthForm.svelte';
</script>

<Seo title="Reset password" noindex />

<AuthShell>
	<AuthForm mode="reset" />
</AuthShell>
