<script lang="ts">
	import Seo from '$lib/components/seo/Seo.svelte';
	import LegalPage from '$lib/components/marketing/LegalPage.svelte';
	import { resolve } from '$app/paths';
</script>

<Seo title="Terms of Service" description="The terms that govern your use of TradeFlex Trading." />

<LegalPage title="Terms of Service" updated="May 1, 2026" href="/legal/terms">
	<p>
		Lorem ipsum dolor sit amet, consectetur adipiscing elit. By accessing or using TradeFlex Trading
		you agree to these terms. If you do not agree, do not use the service. These terms govern your
		use of our website, indicators, courses, and subscription products.
	</p>

	<h2>1. The Service</h2>
	<p>
		Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. TradeFlex Trading provides
		educational content, analytical tools, and software products to active traders and investors.
		Everything we publish is for educational purposes only. <strong
			>No content on this site is personalized investment advice.</strong
		>
	</p>

	<h2>2. Accounts</h2>
	<p>
		Ut enim ad minim veniam, quis nostrud exercitation ullamco. You are responsible for safeguarding
		your account credentials and for any activity that occurs under your account.
	</p>

	<h2>3. Payments & Refunds</h2>
	<p>
		Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore. Subscription
		products renew automatically on the cadence selected at purchase. Indicator and course purchases
		are eligible for a 14-day money-back guarantee. See our <a href={resolve('/legal/refund')}
			>Refund Policy</a
		> for full details.
	</p>

	<h2>4. Acceptable Use</h2>
	<ul>
		<li>Do not redistribute, resell, or reverse-engineer our indicators or course content.</li>
		<li>Do not impersonate TradeFlex or its staff.</li>
		<li>Do not use the service for unlawful purposes.</li>
	</ul>

	<h2>5. Risk Disclosure</h2>
	<p>
		Excepteur sint occaecat cupidatat non proident. Trading involves substantial risk and is not
		suitable for every investor. Past performance is no guarantee of future results.
	</p>

	<h2>6. Limitation of Liability</h2>
	<p>
		To the maximum extent permitted by law, TradeFlex Trading is not liable for any indirect,
		incidental, consequential, special, or exemplary damages, including any loss of profits,
		revenue, data, or trading capital.
	</p>

	<h2>7. Contact</h2>
	<p>
		Questions about these terms? Email <a href="mailto:legal@tradeflextrading.com"
			>legal@tradeflextrading.com</a
		>
		or reach us via the <a href={resolve('/contact')}>Contact page</a>.
	</p>
</LegalPage>
