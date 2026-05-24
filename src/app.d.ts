declare global {
	namespace App {
		interface Error {
			message: string;
			code?: string;
		}
		interface Locals {
			user: {
				id: string;
				email: string;
				name: string;
				role: 'member' | 'admin';
			} | null;
		}
		interface PageData {
			seo?: {
				title?: string;
				description?: string;
				canonical?: string;
			};
		}
		// interface PageState {}
		// interface Platform {}
	}
}

export {};
