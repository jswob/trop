const TOKEN_KEY = 'auth_token';

export const tokenStorage = {
	getToken: () => localStorage.getItem(TOKEN_KEY),
	setToken: (token: string) => localStorage.setItem(TOKEN_KEY, token),
	clearToken: () => localStorage.removeItem(TOKEN_KEY),
	hasToken: () => Boolean(localStorage.getItem(TOKEN_KEY)),
};
