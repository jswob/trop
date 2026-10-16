/** `1 state`, `5 states`. The app and the workbench print a lot of counts and none of them lie. */
export function count(n: number, one: string, many = `${one}s`): string {
	return `${String(n)} ${n === 1 ? one : many}`;
}
