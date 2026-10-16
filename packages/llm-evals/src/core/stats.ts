// The two pieces of statistics the report needs. Both work on cases, never on samples: three
// samples of one case mostly agree, so counting them as three tests would overstate what a run shows.

/** 95% Wilson interval for `hits` out of `n`, as shares in [0, 1]. Holds up for small n and for 0 hits. */
export function wilson(hits: number, n: number, z = 1.96): [number, number] {
	if (n === 0) return [0, 1];
	const p = hits / n;
	const denominator = 1 + (z * z) / n;
	const centre = (p + (z * z) / (2 * n)) / denominator;
	const half = (z * Math.sqrt((p * (1 - p)) / n + (z * z) / (4 * n * n))) / denominator;
	return [Math.max(0, centre - half), Math.min(1, centre + half)];
}

/** A small seeded generator, so the same record always gives the same interval. */
function mulberry32(seed: number): () => number {
	let a = seed >>> 0;
	return () => {
		a = (a + 0x6d2b79f5) >>> 0;
		let t = a;
		t = Math.imul(t ^ (t >>> 15), t | 1);
		t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
		return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
	};
}

export interface Difference {
	mean: number;
	/** 95% interval of the mean, from resampling the cases */
	low: number;
	high: number;
}

/**
 * The mean of per-case differences between two variants on the same cases, with a bootstrap
 * interval. Paired: each case is compared with itself, which removes the noise both variants share.
 */
export function pairedDifference(differences: number[], iterations = 4000, seed = 1): Difference {
	const n = differences.length;
	if (n === 0) return { mean: 0, low: 0, high: 0 };
	const mean = differences.reduce((a, b) => a + b, 0) / n;
	const random = mulberry32(seed);
	const means: number[] = [];
	for (let i = 0; i < iterations; i++) {
		let sum = 0;
		for (let j = 0; j < n; j++) sum += differences[Math.floor(random() * n)] ?? 0;
		means.push(sum / n);
	}
	means.sort((a, b) => a - b);
	return {
		mean,
		low: means[Math.floor(iterations * 0.025)] ?? mean,
		high: means[Math.ceil(iterations * 0.975) - 1] ?? mean,
	};
}

export function median(values: number[]): number {
	if (values.length === 0) return 0;
	const sorted = [...values].sort((a, b) => a - b);
	return sorted[Math.floor(sorted.length / 2)] ?? 0;
}
