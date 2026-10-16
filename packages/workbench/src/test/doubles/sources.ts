/** The double's source: one kit file and the one place that renders it. */
const sources: Record<string, string> = {
	'kit/Mark.tsx': 'export function Mark() {\n\treturn null;\n}\n',
	'views/ledger.tsx': "import { Mark } from '../kit/Mark';\n\nexport const ledger = <Mark />;\n",
};

export default sources;
