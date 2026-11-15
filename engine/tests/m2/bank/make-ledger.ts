// make-ledger.ts: a synthetic bank history 2024-01-01 .. yesterday, standing in for imported, categorised
// transactions. Pay, rent, taxes and the business come from the model's own modelled past (with pay dates
// spread over the window and one December bonus the model does not know about); everyday spending is random
// around 140 zł a day; the past investing is a simple "keep about 12 000, IKE first" habit.
import { assemble, REALITY } from '../load.ts';
import { runModel } from '../engine/run.ts';
import { addDays, isBusinessDay, shiftBusiness } from '../engine/calendar.ts';
import type { Movement } from '../engine/types.ts';

const { model } = assemble(REALITY);
model.ledger = [];
const r = runModel(model, { reality: true });
const places = new Map(model.places.map((p) => [p.id, p]));
let seed = 20261017;
const rnd = () => { seed = (seed * 1664525 + 1013904223) % 4294967296; return seed / 4294967296; };

const rows: Movement[] = [
  { date: '2024-01-01', from: 'otwarcie', to: 'konto', amount: 12000, category: 'otwarcie' },
  { date: '2024-01-01', from: 'otwarcie', to: 'ike', amount: 30000, category: 'otwarcie' },
  { date: '2024-01-01', from: 'otwarcie', to: 'maklerskie', amount: 20000, category: 'otwarcie' },
];
for (const m of r.modelledPast) {
  if (places.get(m.from)?.kind === 'outside' && places.get(m.to)?.kind === 'outside') continue;
  let date = m.date;
  if (m.why === 'my_tax/net') { const d = date.slice(0, 8) + String(3 + Math.floor(rnd() * 5)).padStart(2, '0'); date = shiftBusiness(d, 'prev'); }
  let amount = m.amount;
  if (m.why === 'my_tax/net' && m.period === '2025-11') amount += 3000; // the bonus the model does not know about
  rows.push({ date, from: m.from, to: m.to, amount, category: m.category ?? 'inne', why: 'bank' });
}
for (let d = '2024-01-01'; d < model.today; d = addDays(d, 1)) {
  const n = isBusinessDay(d) ? 1 + Math.floor(rnd() * 3) : Math.floor(rnd() * 3);
  for (let k = 0; k < n; k++) rows.push({ date: d, from: 'konto', to: 'sklepy', amount: Math.round((40 + rnd() * 160) * 100) / 100, category: 'codzienne', why: 'bank' });
  if (rnd() < 0.02) rows.push({ date: d, from: 'konto', to: 'sklepy', amount: Math.round((600 + rnd() * 1400) * 100) / 100, category: 'codzienne', why: 'bank' });
}
rows.sort((a, b) => (a.date < b.date ? -1 : a.date > b.date ? 1 : 0));
// The past habit: on the 15th keep about 12 000 on the account, IKE first up to the limit.
const out: Movement[] = [];
let bal = 0, ikeYear = 0, year = 2024;
const limit = (y: number) => ({ 2024: 23472, 2025: 26019, 2026: 28260 })[y]!;
for (let d = '2024-01-01', i = 0; d < model.today; d = addDays(d, 1)) {
  if (+d.slice(0, 4) !== year) { year = +d.slice(0, 4); ikeYear = 0; }
  while (i < rows.length && rows[i].date === d) {
    const m = rows[i++]; out.push(m);
    if (m.to === 'konto') bal += m.amount; if (m.from === 'konto') bal -= m.amount;
  }
  if (d.slice(8) === '15' && bal > 12000) {
    const x = Math.round((bal - 12000) * 100) / 100, toIke = Math.min(x, limit(year) - ikeYear);
    if (toIke > 0) { out.push({ date: d, from: 'konto', to: 'ike', amount: toIke, category: 'inwestycje', why: 'bank' }); ikeYear += toIke; }
    if (x - toIke > 0) out.push({ date: d, from: 'konto', to: 'maklerskie', amount: Math.round((x - toIke) * 100) / 100, category: 'inwestycje', why: 'bank' });
    bal -= x;
  }
}
await Bun.write(`${import.meta.dir}/ledger.json`, JSON.stringify(out));
console.log(`${out.length} rows, konto on ${addDays(model.today, -1)}: ${bal.toFixed(2)}`);
