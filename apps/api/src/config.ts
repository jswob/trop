export const config = {
	get authPassword() {
		return process.env.AUTH_PASSWORD;
	},
	get jwtSecret() {
		return process.env.JWT_SECRET;
	},
	get webOrigin() {
		return process.env.WEB_ORIGIN;
	},
	get apiPort() {
		return parseInt(process.env.API_PORT || '3001');
	},
};
