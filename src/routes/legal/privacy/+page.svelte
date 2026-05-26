<script lang="ts">
	import Seo from '$lib/components/seo/Seo.svelte';
	import LegalPage from '$lib/components/marketing/LegalPage.svelte';
</script>

<Seo title="Privacy Policy" description="How TradeFlex Trading handles your data." />

<LegalPage title="Privacy Policy" updated="May 1, 2026" href="/legal/privacy">
	<p>
		Lorem ipsum dolor sit amet, consectetur adipiscing elit. This policy describes what information
		we collect, how we use it, and the choices you have. We collect only what is necessary to run
		the service and to improve it.
	</p>

	<h2>Information We Collect</h2>
	<ul>
		<li><strong>Account data</strong> — name, email, hashed password, billing address.</li>
		<li>
			<strong>Usage data</strong> — pages viewed, features used, anonymous performance metrics.
		</li>
		<li>
			<strong>Payment data</strong> — handled by our payment processor; we never store card numbers.
		</li>
		<li><strong>Communications</strong> — emails, support tickets, contact form submissions.</li>
	</ul>

	<h2>How We Use It</h2>
	<p>
		Sed do eiusmod tempor. To operate the service, fulfill your purchases, send transactional
		emails, improve our products, and to detect and prevent fraud or abuse. We do not sell your
		personal data to third parties.
	</p>

	<h2>Your Choices</h2>
	<ul>
		<li>Unsubscribe from marketing emails at any time via the link in any email.</li>
		<li>
			Request a copy of your data by emailing <a href="mailto:privacy@tradeflextrading.com"
				>privacy@tradeflextrading.com</a
			>.
		</li>
		<li>Request deletion of your account from your dashboard or by email.</li>
	</ul>

	<h2>Cookies</h2>
	<p>
		Ut enim ad minim veniam. We use cookies that are essential to the operation of the service
		(login, cart, preferences). Optional analytics cookies are loaded only with your consent.
	</p>

	<h2>Children</h2>
	<p>
		Our service is intended for adults. We do not knowingly collect personal information from anyone
		under 18.
	</p>

	<h2>Contact</h2>
	<p>
		Questions? Email <a href="mailto:privacy@tradeflextrading.com">privacy@tradeflextrading.com</a>.
	</p>
</LegalPage>
