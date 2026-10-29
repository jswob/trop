#!/usr/bin/env python3
"""Write household B's records and documents from fam/b.py's facts and its 2025 simulation.

The export errors of the twist are made here, in Bank Cis's export: October repeats a card row
(same balance after), November's export stops at the 15th, December drops the row of 9 December.

Run: python3 -I tmp/cases/tools/gen_b.py   (story.md, house-rules.md and questions.md are written by hand)
"""
import pathlib
import sys
from datetime import date, timedelta

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))

from fam import b  # noqa: E402
from fam.home import ALL_KNOWN, CAL, ZERO, D, ort  # noqa: E402
from lib.dates import end_of_month  # noqa: E402

ROOT = pathlib.Path(__file__).resolve().parents[1]
OUT = ROOT / "B"
DUP_MONTH, HALF_MONTH, HALF_LAST, DROP_MONTH = 10, 11, date(2025, 11, 15), 12


def c2(x):
    return f"{D(x):.2f}".replace(".", ",")


def sp(x):
    return f"{D(x):,.2f}".replace(",", " ")


def dmy(d, sep="."):
    return d.strftime(f"%d{sep}%m{sep}%Y")


def write(rel, text):
    p = OUT / rel
    p.parent.mkdir(parents=True, exist_ok=True)
    p.write_text(text, encoding="utf-8")


def doc(did, issuer, issued, received, title, body):
    head = f"# {did} · {title}\n\n- id: {did}\n- issuer: {issuer}\n- issued: {issued}\n- received: {received}\n\n"
    write(f"docs/{did}.md", head + body.strip() + "\n")


# --------------------------------------------------------------------------- Bank Cis exports

def q(x):
    return '"' + str(x).replace('"', '""') + '"'


def cis_line(r):
    amt = r["amount"]
    deb = f"{-amt:.2f}" if amt < 0 else ""
    cre = f"{amt:.2f}" if amt > 0 else ""
    return ",".join(q(v) for v in (dmy(r["booked"], "-"), dmy(r["value"], "-"), r["desc"], r["cp"], r["title"], deb, cre, f"{r['balance']:.2f}"))


def cis_head(acct, holder, start, end, opening):
    return [f"# Bank Cis S.A. - account history export", f"# Account: {acct}", f"# Holder: {holder}",
            f"# Period: {dmy(start, '-')} - {dmy(end, '-')}", f"# Opening balance: {opening:.2f}",
            ",".join(q(h) for h in ("Booking date", "Value date", "Type", "Counterparty", "Description", "Debit", "Credit", "Balance"))]


def cis_exports(sim):
    rows = sim["rows"]["cis"]
    made = {}
    for m in range(1, 13):
        start, end = date(2025, m, 1), end_of_month(date(2025, m, 1))
        before = [r for r in rows if r["booked"] < start]
        opening = before[-1]["balance"] if before else b.CIS_OPEN_2025
        mrows = [r for r in rows if r["booked"].month == m]
        out = cis_head(b.PIOTR_CIS, "PIOTR SOWA", start, end, opening)
        lines = []
        for r in mrows:
            if m == HALF_MONTH and r["booked"] > HALF_LAST:
                continue  # the export stops at the 15th
            if m == DROP_MONTH and (r["booked"], -r["amount"], r["desc"]) == b.MISSING_ROW[:3]:
                made["dropped"] = r
                continue
            lines.append(cis_line(r))
            if m == DUP_MONTH and "dup" not in made and r["desc"] == "CARD PAYMENT" and r["booked"] >= date(2025, 10, 15):
                lines.append(cis_line(r))  # the same row twice, the same balance after
                made["dup"] = r
        write(f"records/cis-5521-3380-0001-2025-{m:02d}.csv", "\n".join(out + lines) + "\n")
    return made


def cis_savings(sim):
    out = cis_head(b.PIOTR_SAV, "PIOTR SOWA", date(2025, 1, 1), date(2025, 12, 31), b.SAV_OPEN_2025)
    out += [cis_line(r) for r in sim["rows"]["sav"]]
    write("records/cis-5521-3380-0002-2025.csv", "\n".join(out) + "\n")


def lazur_flr(sim):
    out = ["date,description,amount,balance"]
    for r in sim["rows"]["laz"]:
        desc = r["desc"] if r["desc"] != "incoming transfer" else f"incoming transfer {r['cp']} {r['title']}"
        if r["desc"] == "FX sale":
            desc = r["title"]
        out.append(f"{r['booked']},{desc},{r['amount']:.2f},{r['balance']:.2f}")
    write("records/laz-8813-flr-2025.csv", "\n".join(out) + "\n")


# --------------------------------------------------------------------------- Faktura+ registers

def registers():
    sl, _ = b.sales(known=ALL_KNOWN)  # R1: the generator writes the world, so it sees every figure and rate
    out = ["Faktura+ export;sales register;Piotr Sowa IT services, Kłodzin;2022-04-11..2025-12-31",
           "number;issued;due;client;supply;currency;net;vat rate;vat;gross;vat in DUK;CBL mid;table of"]
    for s in sl:
        rate = "22%" if s["vat"] > 0 else "exempt"
        sup = "IT services" if s["kind"] == "IT" else "sale of a business asset (car LN 4471K)"
        if s["cur"] == "FLR":
            fx = (f"{s['vat_duk']:.2f}", f"{s['mid']}", f"{b.CAL.before(s['issued'])}")
        else:
            fx = ("", "", "")
        out.append(";".join([s["id"], str(s["issued"]), str(s["due"]), s["client"], sup, s["cur"], f"{s['net']:.2f}", rate,
                             f"{s['vat']:.2f}", f"{s['gross']:.2f}", *fx]))
    write("records/faktura-sales-2022-2025.csv", "\n".join(out) + "\n")
    pu = b.purchases()
    out = ["Faktura+ export;purchase register;Piotr Sowa IT services, Kłodzin;2022-04-11..2025-12-31",
           "number;received;supplier;item;net;vat;gross"]
    for p in pu:
        out.append(";".join([p["id"], str(p["received"]), p["supplier"], p["what"], f"{p['net']:.2f}", f"{p['vat']:.2f}", f"{p['gross']:.2f}"]))
    write("records/faktura-purchases-2022-2025.csv", "\n".join(out) + "\n")


# --------------------------------------------------------------------------- Ola's Wrzos statements

def ola_statements():
    for (y, m), st in b.ola_wrzos(known=ALL_KNOWN).items():  # R1: the generator writes the world, so it sees every figure and rate
        last = end_of_month(date(y, m, 1))
        out = ["Bank Wrzos SA;Account history", f"Account;{b.OLA_WRZ}", "Holders;ALEKSANDRA WILK",
               f"Period;{dmy(date(y, m, 1))};{dmy(last)}", f"Opening balance;{c2(st['opening'])}",
               "Booking date;Type;Counterparty;Title;Amount;Balance after"]
        for r in st["rows"]:
            out.append(f"{dmy(r['date'])};{r['type']};{r['cp']};{r['title']};{c2(r['amount'])};{c2(r['balance'])}")
        write(f"records/wrz-ola-{y}-{m:02d}.csv", "\n".join(out) + "\n")


# --------------------------------------------------------------------------- documents

def documents(sim):
    fp = b.flat_purchase()
    rows_car, payoff = b.car_loan()
    legs = b.car_sale_legs()
    mt = sim["mtg"]
    doc("B-D01", "Central Business Register", "2022-04-11", "2022-04-11", "Business register entry: Piotr Sowa", """
Entry no. 22/04/11817. Piotr Sowa, born 3 November 1987, resident of Lenia, address ul. Wrzosowa 12/5, Kłodzin.

Business name: Piotr Sowa IT services. Activity: software development and IT consulting (IT services). Start of activity: 11 April 2022.
""")
    doc("B-D02", "Piotr Sowa", "2022-05-12", "2022-05-12", "Notice to the Revenue Office: form of business tax", """
To the Revenue Office, Kłodzin. I elect the flat rate on revenue (PIT-10 form (c)) for my business income from the tax year 2022.

My first revenue in 2022 was on 29 April 2022. Filed in person, 12 May 2022 (stamp of receipt).
""")
    doc("B-D03", "Social Insurance Office (SIO)", "2022-10-18", "2022-10-18", "SIO: sickness insurance from 1 November 2022", """
Confirmation. Piotr Sowa (business from 11 April 2022) has elected voluntary sickness insurance from 1 November 2022 (SOC-3 .4).
""")
    doc("B-D04", "Notary Ewa Brzoza, Kłodzin", "2022-03-18", "2022-03-18", "Deed of sale: the flat ul. Wrzosowa 12/5 (extract)", f"""
Seller: {b.FLAT_DEED['seller']}, a private person, not a VAT taxpayer. Buyers: Piotr Sowa (a 60 % share) and Aleksandra Wilk (a 40 % share), not married to each other.

1. The flat at {b.FLAT_DEED['address']}, 61.4 m², is sold for {sp(b.FLAT_DEED['price'])} DUK, which the parties state is its market value.
2. Paid today: {sp(b.OWN_FUNDS['Piotr'])} DUK by Piotr Sowa and {sp(b.OWN_FUNDS['Ola'])} DUK by Aleksandra Wilk from their own money; the rest, {sp(b.FLAT_DEED['price'] - b.OWN_FUNDS['Piotr'] - b.OWN_FUNDS['Ola'])} DUK, from the loan of Bank Jodła S.A. no. 7710-22 (both buyers borrowers), paid to the seller today.
3. The notary collected the transfer tax (TT-1) from each buyer on their share: Piotr Sowa {sp(fp['Piotr']['transfer tax'])} DUK, Aleksandra Wilk {sp(fp['Ola']['transfer tax'])} DUK.
4. The notary's fee is {sp(b.FLAT_DEED['notary_fee'])} DUK; each buyer pays it in proportion to their share: {sp(fp['Piotr']['notary'])} DUK and {sp(fp['Ola']['notary'])} DUK, paid today.
5. The buyers take possession today.
""")
    doc("B-D05", "Piotr Sowa", "2022-12-12", "2022-12-12", "Notice to the Revenue Office: VAT", """
I, Piotr Sowa (Piotr Sowa IT services), give up the small-business VAT exemption from 1 January 2023 (VAT-2 .4). I have been charging VAT since my invoice FV 010/2022, with which my 2022 supplies went above the year's limit.
""")
    doc("B-D06", "Arnika Soft sp.", "2022-04-11", "2022-04-11", "Arnika Soft · contract for IT services (extract, with its annexes)", """
Contractor: Piotr Sowa IT services. Client: Arnika Soft sp., Dębowo.

1. The contractor provides software development services each month.
2. The fee is monthly: for April 2022, 9 000.00 DUK net; then 18 000.00 DUK net a month. VAT is added where the contractor charges it.
3. The contractor invoices on the last business day of each month; the client pays within 14 days of the invoice.

Annex 1 (2022-12-14): from January 2023 the fee is 20 000.00 DUK net a month.
Annex 2 (2023-12-13): from January 2024 the fee is 22 000.00 DUK net a month.
Annex 3 (2024-12-16): from January 2025 the work is reduced; the fee is 16 000.00 DUK net a month.
""")
    doc("B-D07", "Bank Jodła S.A.", "2022-03-10", "2022-03-10", "Bank Jodła · mortgage loan agreement 7710-22 (extract)", f"""
Borrowers: Piotr Sowa and Aleksandra Wilk, jointly. Purpose: buying the flat at {b.FLAT_DEED['address']}, their home.

1. Amount: {sp(b.MTG_PRINCIPAL)} DUK, paid to the seller on the day of the deed. Term: {b.MTG_N} monthly instalments, on the 5th of each month, the first on 5 April 2022. An instalment day that isn't a business day moves to the next business day; the interest is counted to the 5th.
2. Rate: DUKOR 6M plus a margin of {b.MTG_MARGIN} percentage points, a year. DUKOR 6M is reset on 1 January and 1 July: the fixing of the last business day before the reset day applies until the next reset. The first rate is set from the fixing of the last business day before the payment day.
3. Interest is charged daily on the day's balance at the rate in force that day, over 365 days, from the payment day. The interest of an instalment is what accrued since the last instalment day (for the first, since the payment day), rounded to the ort.
4. Instalments are equal (annuity). The instalment is set again, for the instalments left, at the first instalment after each reset and after each overpayment, so that the loan ends on the same day.
5. Overpayment: allowed on any day. A fee of 2 % of the amount overpaid is charged on overpayments made within 36 months of the payment day; then none. The fee is taken from the amount overpaid; the rest reduces the balance on the day.
6. Payment holidays under the law are granted on notice (MTG-1).
""")
    notices = (date(2022, 8, 1), date(2022, 8, 22), date(2022, 10, 24), date(2022, 11, 21), date(2023, 1, 23))
    hol = "\n".join(f"- notice of {dmy(n)}: the instalment of {dmy(h)} is suspended." for n, h in zip(notices, b.HOLIDAYS))
    doc("B-D08", "Bank Jodła S.A.", "2023-01-24", "2023-01-24", "Bank Jodła · payment holidays granted on loan 7710-22", f"""
Under MTG-1, at the borrowers' notices, these instalments are suspended; nothing is paid for them, no interest is charged for those months, and every later instalment moves by one month.

{hol}
""")
    doc("B-D09", "Housing association WM Wrzosowa 12", "2024-12-10", "2024-12-12", "WM Wrzosowa 12 · the building fee for 2025", f"""
Flat 12/5 (owners Piotr Sowa and Aleksandra Wilk). The monthly fee for 2025 is {sp(b.FLAT_FEE)} DUK, due by the 10th of each month.
""")
    doc("B-D10", "TU Bocian S.A.", "2025-03-10", "2025-03-10", "TU Bocian · flat insurance policy WRZ-12/5 2025/26", f"""
Insured: the flat ul. Wrzosowa 12/5. Policyholders: Piotr Sowa and Aleksandra Wilk. Cover 15 March 2025 .. 14 March 2026. Premium {sp(b.FLAT_INS_2025[1])} DUK, paid on {dmy(b.FLAT_INS_2025[0])} by Aleksandra Wilk.
""")
    doc("B-D11", "Auto Grab sp.", "2023-03-15", "2023-03-15", "Auto Grab · invoice FA/2023/0311: the car LN 4471K", """
Buyer: Piotr Sowa IT services. A Skoda Octavia, engine 1 998 cm³, registered LN 4471K on 15 March 2023 to Piotr Sowa.

Net 80 000.00 DUK, VAT 22 % 17 600.00 DUK, gross 97 600.00 DUK. Paid: 37 600.00 DUK from the buyer's account, 60 000.00 DUK from Bank Cis's loan 5521-K.

(Note in Piotr's hand: entered in the business's asset register on 15 March 2023; used in the business and privately, no mileage log.)
""")
    doc("B-D12", "Bank Cis S.A.", "2023-03-14", "2023-03-14", "Bank Cis · car loan agreement 5521-K (extract)", f"""
Borrower: Piotr Sowa. A consumer credit.

1. Amount: {sp(b.CAR_LOAN['principal'])} DUK, paid to Auto Grab sp. on 15 March 2023.
2. Interest: fixed, {b.CAR_LOAN['rate'] * 100:.1f} % a year; a month's interest is the balance times {b.CAR_LOAN['rate'] * 100:.1f} % / 12, rounded to the ort.
3. {b.CAR_LOAN['n']} equal instalments of {sp(payoff['instalment'])} DUK on the 15th of each month, the first on 15 April 2023; the last settles what is left. An instalment day that isn't a business day moves to the next business day.
4. On repayment between instalment days, interest for the days since the last instalment day: the balance times {b.CAR_LOAN['rate'] * 100:.1f} % times the days / 365, rounded to the ort.
5. Early repayment: allowed at any time. Fee: 1 % of the amount repaid early.
""")
    ins = b.CAR_INS_2025
    doc("B-D13", "TU Bocian S.A.", "2025-03-14", "2025-03-14", "TU Bocian · car liability policy MTPL 88-4471 2025/26", f"""
Car LN 4471K. Holder: Piotr Sowa. Cover {dmy(ins['from'])} .. {dmy(ins['to'])}. Premium {sp(ins['premium'])} DUK, paid {dmy(ins['paid'])}.

Terms §9: when the policy ends early, the insurer refunds the premium times the days left (from the day after the end to {dmy(ins['to'])}) over the policy's days, rounded down to the ort, less a handling fee of {sp(ins['fee'])} DUK.
""")
    doc("B-D14", "Gmina Kłodzin", "2025-02-17", "2025-02-17", "Gmina Kłodzin · road tax 2025, car LN 4471K", f"""
Holder: Piotr Sowa. Engine 1 998 cm³. Road tax for 2025: {sp(b.ROAD_TAX)} DUK, due by 31 March 2025.
""")
    doc("B-D15", "Piotr Sowa IT services", "2025-09-12", "2025-09-12", "Invoice FV of the car sale", f"""
Seller: Piotr Sowa IT services. Buyer: {b.CAR_SALE[1]}, Kłodzin. The car LN 4471K, sold and handed over on {dmy(b.CAR_SALE[0])}.

Net {sp(legs['net'])} DUK, VAT 22 % {sp(legs['vat'])} DUK, gross {sp(legs['price'])} DUK, paid by transfer on {dmy(b.CAR_SALE[0])}. (The invoice's number is in the sales register.)
""")
    doc("B-D16", "Bank Cis S.A.", "2025-09-16", "2025-09-16", "Bank Cis · loan 5521-K repaid early", f"""
On {dmy(payoff['date'])} the borrower repaid the loan in full from account {b.PIOTR_CIS}:

- balance after the instalment of 15 September 2025: {sp(payoff['balance'])} DUK
- interest for {payoff['days']} day: {sp(payoff['interest'])} DUK
- early repayment fee: {sp(payoff['fee'])} DUK (0.5 % of the amount repaid: less than a year was left to the agreed end, {dmy(payoff['agreed end'])}, CC-2 .2)
- total taken: {sp(payoff['total'])} DUK

The loan is closed.
""")
    doc("B-D17", "TU Bocian S.A.", "2025-09-25", "2025-09-26", "TU Bocian · policy MTPL 88-4471 ended", f"""
The car LN 4471K was sold on {dmy(b.CAR_SALE[0])}; the policy ended that day (MOT-1 .2). The refund under §9 is paid to the holder's account on 29 September 2025.
""")
    doc("B-D18", "Arnika-Delta Group sp.", "2025-04-08", "2025-04-09", "Arnika-Delta · merger notice", """
From 1 May 2025 Arnika Soft sp. is merged into Arnika-Delta Group sp., which takes over all its contracts. Please invoice Arnika-Delta Group sp. from May 2025; payments will come from our account under the new name.
""")
    doc("B-D19", "Arnika-Delta Group sp. (e-mail, accounts payable)", "2025-11-14", "2025-11-14", "Arnika-Delta · the recalled payment", f"""
Dear Mr Sowa, our bank recalled by mistake our payment of 12 November for your October invoice. We are sorry. We will pay it on {dmy(b.ARNIKA_REPAY)}, with the payment run of that day.
""")
    doc("B-D20", "Studio Bez (e-mail, Wanda Bez)", "2025-12-29", "2025-12-29", "Studio Bez · the rest of the June invoice", """
Hi Piotr, about the rest of your June invoice: we'll probably pay it next week, once our client pays us. Thanks for waiting. Wanda
""")
    doc("B-D21", "Kessel Lenia sp.", "2025-01-08", "2025-01-08", "Kessel Lenia · contract for IT services (extract)", f"""
Contractor: Piotr Sowa IT services. Client: Kessel Lenia sp., Dębowo. Term: January 2025 .. December 2025; it ends then.

1. The fee is {sp(b.KESSEL_NET_FLR)} FLR net a month, VAT added. The contractor invoices in florens on the last business day of the month.
2. The client pays within 30 days, in florens, into the contractor's floren account at Bank Lazur, {b.PIOTR_LAZ}.
""")
    doc("B-D22", "Kessel Lenia sp. (e-mail, finance)", "2025-12-30", "2025-12-30", "Kessel · the November invoice", """
Dear Piotr, your November invoice is approved; it will be paid in the first days of January, probably on the 5th.
""")
    doc("B-D23", "Bank Lazur", "2025-01-02", "2025-01-02", "Bank Lazur · floren account LN88 1020 0044 8813 FLR: terms (extract)", """
Holder: Piotr Sowa. An account in florens; no interest, no fees.

§4. A sale of florens from the account to an account in dukats is made at Bank Lazur's buy rate of the day of the order (the exchange table), the dukats rounded to the ort, half up, and credited to the other bank's account on the next business day.
""")
    doc("B-D24", "Bartłomiej Kos", "2025-07-01", "2025-07-01", "Note from Bartek", f"""
Piotr, I borrowed {sp(b.BARTEK['lent'][1])} DUK from you on {dmy(b.BARTEK['lent'][0])}. Today I sent you {sp(b.BARTEK['back'][1])} DUK. I will pay the rest on {dmy(b.BARTEK['promised'])}. No interest, as we agreed. Bartek
""")
    for y, did in ((2024, "B-D25"), (2025, "B-D25b")):
        doc(did, "Mewa Logistyka sp.", f"{y - 1}-12-16", f"{y - 1}-12-16", f"Mewa Logistyka · pay from January {y}", f"""
Dear Aleksandra, from January {y} your monthly gross pay is {sp(b.OLA_GROSS[y])} DUK. Pay is paid on the 28th of the month it is for; where the 28th is not a business day, on the last business day before it. You have filed the statement for the yearly reducing amount (PIT-4); it stays on file.
""")
    doc("B-D26", "Mewa Logistyka sp.", "2025-02-27", "2025-03-03", "Mewa Logistyka · February's pay is late", """
Dear employees, because of a change of our payroll system, the pay for February 2025 will be paid on 7 March 2025. We apologise. (Ola read the letter on 3 March.)
""")
    doc("B-D27", "Notary Ewa Brzoza, Kłodzin", "2025-06-13", "2025-06-16", "Notary · estate of Zbigniew Sowa: payout to Piotr Sowa", f"""
From the estate of Zbigniew Sowa (died 2024), held on the notary's deposit account: {sp(b.ESTATE_PAYOUT[1])} DUK paid to Piotr Sowa, an heir, on {dmy(b.ESTATE_PAYOUT[0])}, to account {b.PIOTR_SAV}.
""")
    pays = []
    for r in mt.rows:
        if not r["holiday"] and r["pay_day"].year == 2025:
            pays.append((r["pay_day"], f"instalment of {dmy(r['slot'])}", r["instalment"], None))
    for o in mt.overpays:
        if o["date"].year == 2025:
            pays.append((o["date"], "overpayment", o["paid"], o["fee"]))
    pays.sort(key=lambda x: x[0])
    lines = "\n".join(f"| {dmy(d)} | {what} | {sp(a)} | {'' if f is None else sp(f)} |" for d, what, a, f in pays)
    doc("B-D28", "Bank Jodła S.A.", "2026-01-07", "2026-01-07", "Bank Jodła · loan 7710-22: payments received in 2025", f"""
Borrowers: Piotr Sowa and Aleksandra Wilk. All payments came from account {b.PIOTR_CIS}.

| booked | what | amount, DUK | of it, the fee |
|---|---|---|---|
{lines}
""")
    doc("B-D29", "Bank Jodła S.A.", "2024-11-05", "2024-11-08", "Bank Jodła · valuation of the flat ul. Wrzosowa 12/5", f"""
For the review of loan 7710-22, the bank's valuer values the flat on {dmy(b.VALUATION[0])} at {sp(b.VALUATION[1])} DUK.
""")
    doc("B-D30", "Piotr Sowa and Aleksandra Wilk", "2025-01-03", "2025-01-03", "Our note: the flat's costs settled to the end of 2024", """
We have gone through the flat's costs from the deed to 31 December 2024: what each of us paid against our shares. We have settled the difference in cash today. Nothing is owed between us for the flat up to 31 December 2024. Piotr, Ola
""")
    doc("B-D31", "Halina Dąb (landlady)", "2019-08-26", "2019-08-26", "Lease of the flat ul. Lipowa 4/7, Kłodzin (extract)", f"""
Tenant: Piotr Sowa. From {dmy(b.RENT['from'])}, for an indefinite time.

1. Rent: {sp(b.RENT['monthly'])} DUK a month, paid in advance by the 1st of the month.
2. Deposit: {sp(b.RENT['deposit'])} DUK, paid on signing; returned within 30 days after the flat is handed back, less the cost of any damage.
3. Either side may end the lease by written notice of three months, to the end of a calendar month.
""")
    doc("B-D32", "Halina Dąb (landlady)", "2022-07-15", "2022-07-15", "Ul. Lipowa 4/7 · the lease ended, the deposit settled", f"""
Mr Sowa's notice of {dmy(b.RENT['notice'])} ended the lease on {dmy(b.RENT['ends'])}; the rent was paid to the end. The flat was handed back on 30 June 2022. From the deposit of {sp(b.RENT['deposit'])} DUK I keep {sp(b.RENT['kept'])} DUK for repainting the hall; I return {sp(b.RENT['returned'][1])} DUK today, {dmy(b.RENT['returned'][0])}, to Mr Sowa's account.
""")
    s = b.STUDIO
    doc("B-D33", "Notary Adam Kłos, Dębowo", str(s["bought"]), str(s["bought"]), "Deed of sale: the studio ul. Polna 3/14, Dębowo (extract)", f"""
Buyer: Aleksandra Wilk, alone. Price {sp(s['price'])} DUK. Transfer tax collected {sp(s['tt'])} DUK; the notary's fee {sp(s['notary'])} DUK, paid by the buyer.
""")
    doc("B-D34", "Notary Adam Kłos, Dębowo", str(s["sold"]), str(s["sold"]), "Deed of sale: the studio ul. Polna 3/14 sold (extract), with the agent's invoice", f"""
Seller: Aleksandra Wilk. Price {sp(s['sale_price'])} DUK, paid today. The agent Domy Lenia sp. invoices her {sp(s['agent'])} DUK for the sale (invoice 2021/11/88, paid 2021-11-24).
""")
    doc("B-D35", "Kuchnie Buk sp.", str(b.KITCHEN_QUOTE[0]), str(b.KITCHEN_QUOTE[0]), "Kuchnie Buk · quote for a fitted kitchen, ul. Wrzosowa 12/5", f"""
For Piotr Sowa and Aleksandra Wilk. A fitted kitchen with appliances: {sp(b.KITCHEN_QUOTE[1])} DUK gross, fitting in spring 2023 on order. The quote holds to 31 December 2022.
""")
    doc("B-D36", "Bank Cis S.A.", "2024-11-20", "2024-11-20", "Bank Cis · fees for account LN45 2090 0017 5521 3380 0001 from 2025", """
Holder: Piotr Sowa. The account is free of monthly fees. The debit card ...3380: a yearly card fee of 249.00 DUK, charged on 9 December (on the next business day where the 9th isn't one).
""")


def main():
    sim = b.simulate(True, known=ALL_KNOWN)  # R1: the generator writes the world, so it sees every figure and rate
    made = cis_exports(sim)
    cis_savings(sim)
    lazur_flr(sim)
    registers()
    ola_statements()
    documents(sim)
    print("B written; duplicate", made["dup"]["booked"], made["dup"]["amount"], "; dropped", made["dropped"]["booked"], made["dropped"]["amount"])


if __name__ == "__main__":
    main()
