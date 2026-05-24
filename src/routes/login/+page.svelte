<script lang="ts">
	import Seo from '$lib/components/seo/Seo.svelte';
	import AuthShell from '$lib/components/layout/AuthShell.svelte';
	import AuthForm from '$lib/components/forms/AuthForm.svelte';
</script>

<Seo title="Sign in" description="Sign in to your TradeFlex Trading account." noindex />

<AuthShell>
	<AuthForm mode="login" />
</AuthShell>
