import createClient from 'openapi-fetch';
import type { paths, components } from '$lib/generated/openapi-schema';
import { error } from '@sveltejs/kit';

// TODO: Configurable
const BASE_URL = 'http://localhost:8080/';
const CLIENT = createClient<paths>({
	baseUrl: BASE_URL,
	querySerializer: {
		object: {
			style: 'form',
			explode: true,
		},
	},
});

export type Post = components['schemas']['Post'];
export type PaginationReference = components['schemas']['PaginationReference'];
export type Oauth2ProviderChoice = components['schemas']['Oauth2ProviderChoice'];
export type AuthUrlResponse = components['schemas']['AuthUrlResponse'];
export type AuthTokenResponse = components['schemas']['AuthTokenResponse'];

export async function getPost(id: string): Promise<Post | undefined> {
	const response = await CLIENT.GET('/posts/{id}', {
		params: { path: { id } },
	});

	if (!response.data) {
		if (response.response.status !== 404) {
			error(500);
		}
	}

	return response.data;
}

export async function getRecentPosts(
	perPage: number,
	pagination_reference: PaginationReference,
): Promise<Post[]> {
	const response = await CLIENT.GET('/posts/recent', {
		params: {
			query: { per_page: perPage, pagination_reference },
		},
	});

	if (!response.data) {
		error(500);
	}

	return response.data;
}

export async function getAuthUrl(
	provider: Oauth2ProviderChoice,
	redirect: URL,
	session_id: string,
): Promise<AuthUrlResponse> {
	const response = await CLIENT.GET('/oauth2/auth-url', {
		params: {
			query: {
				provider,
				redirect: redirect.toString(),
				session_id,
			},
		},
	});

	if (!response.data) {
		error(500);
	}

	return response.data;
}

export async function getToken(
	code: string,
	csrf_token: string,
	session_id: string,
	expires: boolean,
): Promise<AuthTokenResponse> {
	const response = await CLIENT.GET('/oauth2/get-token', {
		params: {
			query: {
				code,
				csrf_token,
				session_id,
				expires,
			},
		},
	});

	if (!response.data) {
		error(500);
	}

	return response.data;
}
