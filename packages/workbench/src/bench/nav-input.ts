import { setup } from '../lib/bound';
import { groundView, viewsOfSection } from '../lib/registry';
import type { PlaceSubject } from '../lib/setup';

import type { NavInput, NavPlace } from './nav';

const place = (view: PlaceSubject): NavPlace => ({
	id: view.id,
	title: view.title,
	states: view.variants.length,
});

/** What this workbench registers, as the sidebar and the jump list read it. */
export function benchNav(): NavInput {
	const ground = groundView();

	return {
		...(ground ? { ground: { ...place(ground), label: setup.groundLabel ?? ground.title } } : {}),
		destinations: setup.sections.map(section => ({
			id: section.id,
			label: section.label,
			places: viewsOfSection(section.id).map(place),
		})),
		kit: setup.kit.entries.map(entry => ({ id: entry.id, name: entry.name })),
		protos: setup.protos.map(proto => ({ id: proto.id, title: proto.title })),
	};
}
