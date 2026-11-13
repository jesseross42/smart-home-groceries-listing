/** Offers the credential separately so the server can select only the event protocol. */
export function socketProtocols(token: string | null): string[] {
	return token
		? ["grocery.events", `grocery.session.${token}`]
		: ["grocery.events"];
}
