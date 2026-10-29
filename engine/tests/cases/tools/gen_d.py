#!/usr/bin/env python3
"""Write household D's records and documents from fam/d.py's facts and its simulation.

Bank Olcha's export errors are made here, in the export only: February 2026 repeats the tenant's
rent row (the balance after counts it once); March 2026 leaves out the row of the building fee of
5 March (the balance after counts it).

Run: python3 -I tmp/cases/tools/gen_d.py   (story.md, house-rules.md and questions.md are written by hand)
"""
import pathlib
import sys
from datetime import date

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))

from fam import d as F  # noqa: E402
from fam import e as E  # noqa: E402
from fam.home import ALL_KNOWN, ZERO, D, figure  # noqa: E402
from lib.dates import end_of_month  # noqa: E402

ROOT = pathlib.Path(__file__).resolve().parents[1]
OUT = ROOT / "D"
DUP = (2026, 2)      # the repeated row
DROP = (2026, 3)     # the missing row


def amt(x):
    s = f"{abs(D(x)):,.2f}".replace(",", " ").replace(".", ",")
    return ("-" if D(x) < 0 else "") + s


def sp(x):
    return f"{D(x):,.2f}".replace(",", " ")


def write(rel, text):
    p = OUT / rel
    p.parent.mkdir(parents=True, exist_ok=True)
    p.write_text(text, encoding="utf-8")


def doc(did, issuer, issued, received, title, body):
    head = f"# {did} · {title}\n\n- id: {did}\n- issuer: {issuer}\n- issued: {issued}\n- received: {received}\n\n"
    write(f"docs/{did}.md", head + body.strip() + "\n")


# --------------------------------------------------------------------------- Bank Olcha

def olcha(sim):
    rows = sim["rows"]
    bal = sim["opening"]
    y, m = 2024, 1
    while (y, m) <= (2026, 9):
        start, end = date(y, m, 1), end_of_month(date(y, m, 1))
        mine = [r for r in rows if start <= r["date"] <= end]
        holder = "ZBIGNIEW SOWA, HALINA SOWA" if (y, m) < (2024, 6) else "HALINA SOWA"
        out = ["Bank Olcha S.A.|account statement", f"Account|{F.ACCT}", f"Holder|{holder}",
               f"Period|{start:%Y%m%d}|{end:%Y%m%d}", f"Opening balance|{amt(bal)}",
               "Date|Type|Counterparty|Title|Amount|Balance after"]
        for r in mine:
            line = "|".join([r["date"].strftime("%Y%m%d"), r["type"], r["cp"], r["title"], amt(r["amount"]), amt(r["balance"])])
            if (y, m) == DROP and r["tag"] == "fee":
                bal = r["balance"]
                continue
            out.append(line)
            if (y, m) == DUP and r["tag"] == "rent":
                out.append(line)
            bal = r["balance"]
        out.append(f"Closing balance|{amt(bal)}")
        write(f"records/olcha-{y}-{m:02d}.csv", "\n".join(out) + "\n")
        y, m = (y + 1, 1) if m == 12 else (y, m + 1)


def kopia():
    g = E.GIFT                             # the gift, stated once (fam/e.py)
    n = sum((u for _, _, u in g["lots"]), D(0))
    out = [f"DM Kopia\tsecurities account {g['from']} (HALINA SOWA)\tstatement 01.01.2023 - 31.10.2023\tclosed 31.10.2023",
           "Date\tEvent\tInstrument\tQty\tNote",
           f"2023-01-01\tOPENING\t{g['fund']}\t{n}\tunits held",
           f"{g['day'].isoformat()}\tTRANSFER OUT (gift)\t{g['fund']}\t-{n}\tto {g['into']} (ANNA LIS)",
           f"2023-10-31\tCLOSING\t{g['fund']}\t0\taccount closed"]
    write("records/kopia-30-118-117-halina.tsv", "\n".join(out) + "\n")


# --------------------------------------------------------------------------- documents

def documents(sim):
    div = sim["division"]
    c = F.death_day_balance(known=ALL_KNOWN)  # R1: the generator writes the world, so it sees every figure
    doc("D-D01", "Civil Registry Office, Kłodzin", "2024-05-20", "2024-05-20", "Death certificate: Zbigniew Sowa (extract)", f"""
Zbigniew Sowa, born 20.11.1953, died 17.05.2024 in Kłodzin. Married to Halina Sowa (née Kos), born 08.03.1959.
""")
    doc("D-D02", "Civil Registry Office, Kłodzin", "2024-05-20", "2024-05-20", "Marriage certificate (extract)", """
Zbigniew Sowa and Halina Kos married on 14.06.1980 in Kłodzin. No marital contract is noted.
""")
    doc("D-D03", "Civil Registry Office, Kłodzin", "2024-06-03", "2024-06-03", "Extracts: Adam Sowa and Jakub Sowa", """
Adam Sowa, born 02.04.1981, son of Zbigniew and Halina Sowa, died 30.09.2016.
Jakub Sowa, born 11.02.2004, son of Adam Sowa. Adam Sowa had no other children.
""")
    doc("D-D04", "District Court, Kłodzin", "2024-10-17", "2024-10-21", "Certificate of inheritance (extract)", """
The estate of Zbigniew Sowa, who died on 17 May 2024 and left no will, is inherited by law (CIV-3) by: his wife Halina Sowa; his children Piotr Sowa, Anna Lis and Marta; and, in place of his son Adam Sowa, who died before him, Adam's son Jakub Sowa.

The certificate becomes final on 3 November 2024.
""")
    doc("D-D05", "Notary Ewa Brzoza, Kłodzin", "2024-07-08", "2024-07-08", "Inventory of the estate of Zbigniew Sowa (extract)", f"""
As the heirs state it, with the values they give:

| item | whose before the death | value given |
|---|---|---|
| the flat ul. Ogrodowa 2/11, Kłodzin (D-D06) | Zbigniew's own | 230 000.00 DUK (agent's estimate, Domy Lenia sp., 2024-07-02, for the day of the death) |
| 325 treasury savings bonds, register 55-1904-2271 (D-D22) | community (bought from the joint account) | at their value on the day of the death |
| Bank Olcha account {F.ACCT} | community | {sp(c)} DUK at the end of 17 May 2024 (D-D33) |
| Bank Buk loan 3310-20 (D-D18) | community debt | {sp(F.LOAN_AT_DEATH)} DUK principal outstanding on 17 May 2024; paid off by TU Opoka on 12 July 2024 (D-D20) |

The house ul. Leśna 4, Brzezie, is Halina's own (inherited from her parents in 1998) and is not in the estate.
""")
    doc("D-D06", "Land Registry, Kłodzin", "2024-07-01", "2024-07-02", "Land register extract: ul. Ogrodowa 2/11, Kłodzin", """
Flat no. 11, ul. Ogrodowa 2, Kłodzin, 31.4 m². Owner: Zbigniew Sowa, alone, by inheritance from his mother Stanisława Sowa (died 2 December 2011; certificate of inheritance of 15 March 2012). She had owned it since 1976.
""")
    p = div
    doc("D-D07", "Notary Ewa Brzoza, Kłodzin", "2025-06-12", "2025-06-12", "Agreement dividing the estate of Zbigniew Sowa (extract)", f"""
Parties: Halina Sowa, Piotr Sowa, Anna Lis, Marta, Jakub Sowa, the heirs under the certificate of 17 October 2024.

1. Halina Sowa takes: the shares of Piotr Sowa, Anna Lis and Marta in the flat ul. Ogrodowa 2/11; the estate's half of the 325 bonds in register 55-1904-2271; the estate's half of the Bank Olcha account {F.ACCT}; the refund on Zbigniew's 2024 return ({sp(F.ZB_REFUND)} DUK, paid into that account on 21 May 2025). Jakub Sowa keeps his share in the flat. From today the flat belongs to Halina Sowa 13/16 and Jakub Sowa 3/16.
2. The parties value: the flat at 165 000.00 DUK; the bonds at their value today; the account at its balance at the end of 17 May 2024 ({sp(c)} DUK).
3. Halina Sowa paid the flat's building fees from 18 May 2024 to today ({sp(p['costs'])} DUK, {len(F.estate_flat_costs())} payments). Each other heir's share of them is set off against what Halina Sowa pays that heir.
4. Halina Sowa pays each heir the value of what that heir gives up, less the set-off in point 3, rounded down to whole dukats; what is rounded off stays with her. Of each payment to Piotr Sowa, Anna Lis and Marta, 30 937.50 DUK is for the share in the flat.
5. Halina Sowa pays the whole sum into the notary's deposit account today. The notary pays it out on 16 June 2025: to Piotr Sowa to {F.PIOTR_ACCT}, to Anna Lis to {F.ANIA_ACCT}, to Marta to her account, and to Jakub Sowa to {F.KUBA_OLD}.
6. No transfer tax is due: this is a division between heirs, not a sale (TT-1). The notary's fee for this deed, 1 845.00 DUK, was paid by Halina Sowa in cash today; 1 230.00 DUK of it is for the transfer of the shares in the flat.
""")
    doc("D-D08", "Halina Sowa", "2025-05-05", "2025-05-05", "Report of an acquisition by inheritance to the Revenue Office (copy)", """
Halina Sowa reports the acquisition, by inheritance from her husband Zbigniew Sowa (died 17.05.2024), of her share in his estate under the certificate of the District Court, Kłodzin, final on 3 November 2024. Filed at the Revenue Office Kłodzin on 5 May 2025 (stamp on the copy).
""")
    doc("D-D09", "Social Insurance Office (SIO), Kłodzin", "2018-11-28", "2018-11-30", "SIO decision: old-age pension, Zbigniew Sowa", f"""
From 1 December 2018. Capital on retirement {sp(F.ZB_CAPITAL)} DUK (SOC-5.3); LE-65 206.1 months (table 2018); the pension is {sp(F.ZB_INITIAL)} DUK a month (SOC-5.4). Paid on the 5th of each month, or on the last business day before it, to the account {F.ACCT}.
""")
    hist = "\n".join(f"| {y} | {sp(e)} |" for y, e in sorted(F.H_EARNINGS.items()))
    doc("D-D10", "Social Insurance Office (SIO), Kłodzin", "2019-06-12", "2019-06-14", "SIO statement of earnings: Halina Sowa, 1981–2018", f"""
The earnings on which contributions were due, as the SIO's records show them:

| year | earnings, DUK |
|---|---|
{hist}
""")
    # R1: the generator writes the world, so it sees every figure
    doc("D-D11", "Social Insurance Office (SIO), Kłodzin", "2019-06-24", "2019-06-27", "SIO decision: old-age pension, Halina Sowa", f"""
On the claim of 10 June 2019: from 1 July 2019. Capital on retirement {sp(F.capital(F.H_EARNINGS, 2019, known=ALL_KNOWN))} DUK; LE-60 261.0 months (table 2019); the pension is {sp(F.H_INITIAL)} DUK a month. Paid on the 5th of each month, or on the last business day before it, to the account {F.ACCT}.
""")
    doc("D-D12", "Halina Sowa", "2024-05-27", "2024-05-27", "Claim for a survivor pension (copy)", """
Halina Sowa, widow of Zbigniew Sowa (died 17.05.2024), claims the survivor pension (SOC-7) and elects to be paid the survivor pension in place of her own old-age pension. Filed at the SIO, Kłodzin, on 27 May 2024.
""")
    # R1: the generator writes the world, so it sees every figure
    doc("D-D13", "Social Insurance Office (SIO), Kłodzin", "2024-06-14", "2024-06-17", "SIO decision: survivor pension, Halina Sowa", f"""
Survivor pension from 1 June 2024: 85 % of the pension of Zbigniew Sowa on 17 May 2024 ({sp(F.zb(F.DEATH, known=ALL_KNOWN))} DUK), {sp(F.SURV_BASE)} DUK a month, indexed as his pension would have been (SOC-7 .1). As elected, it is paid in place of the old-age pension (SOC-7 .3). For June 2024 the difference to the old-age pension already paid on 5 June is paid on 24 June 2024.
""")
    doc("D-D14", "Halina Sowa", "2025-08-26", "2025-08-26", "Claim under SOC-7 v2 (copy)", """
Halina Sowa claims to be paid, beside her survivor pension (elected in full), the part of her old-age pension under SOC-7 v2 point 3. Filed at the SIO, Kłodzin, on 26 August 2025.
""")
    # R1: the generator writes the world, so it sees every figure
    sur25, own25 = F.surv(date(2025, 7, 1), known=ALL_KNOWN), F.own(date(2025, 7, 1), known=ALL_KNOWN)
    # R1: the generator writes the world, so it sees every figure
    doc("D-D15", "Social Insurance Office (SIO), Kłodzin", "2025-11-20", "2025-11-24", "SIO decision: the survivor pension with a part of the old-age pension", f"""
On the claim of 26 August 2025, from July 2025: the survivor pension {sp(sur25)} DUK is paid in full, and a part of the old-age pension ({sp(own25)} DUK): 15 % is {sp(D('0.15') * own25)} DUK. Both together may not exceed 3 times the minimum pension, 3 × 1 780.96 = 5 342.88 DUK (the minimum pension in force on the day of the death); the part is cut to {sp(F.part_sio(date(2025, 7, 1), known=ALL_KNOWN))} DUK.

The parts for July–November 2025, {sp(5 * F.part_sio(date(2025, 7, 1), known=ALL_KNOWN))} DUK, are paid as arrears with the payment of 5 December 2025.
""")
    for yr, did, rec in ((2024, "D-D16", "2025-02-14"), (2025, "D-D17", "2026-02-13")):
        pays = sim["sio"]["H"][yr]
        inc = sum((x["gross"] for x in pays), ZERO)
        adv = sum((x["advance"] for x in pays), ZERO)
        hl = sum((x["health"] for x in pays), ZERO)
        doc(did, "Social Insurance Office (SIO), Kłodzin", rec, rec, f"SIO yearly statement for {yr} (PIT): Halina Sowa", f"""
Pensions paid in {yr}: {sp(inc)} DUK. Health contributions deducted: {sp(hl)} DUK. Tax advances withheld: {sp(adv)} DUK. A reducing statement was on file for the whole year.
""")
    doc("D-D18", "Bank Buk S.A.", "2020-09-10", "2020-09-10", "Bank Buk · mortgage loan agreement 3310-20 (extract)", """
Borrowers: Zbigniew Sowa and Halina Sowa, jointly. 90 000.00 DUK, paid out on 15 September 2020, for the renovation of the house ul. Leśna 4, Brzezie, secured on it. Fixed rate 5.40 % a year; interest each month on the balance at 0.45 %, to the ort. 120 equal instalments, the first on 5 October 2020, then on the 5th of each month, or the next business day; collected from the account at Bank Olcha.
Borrower insurance: TU Opoka S.A., policy KR-3310-20, on the life of Zbigniew Sowa (D-D19).
""")
    doc("D-D19", "TU Opoka S.A.", "2020-09-10", "2020-09-10", "TU Opoka · borrower life insurance KR-3310-20 (extract)", """
On the death of the insured, TU Opoka pays Bank Buk the loan's principal outstanding on the day of the death. The bank charges no interest after the day of the death and returns what it collected after it.
""")
    doc("D-D20", "Bank Buk S.A.", "2024-07-16", "2024-07-18", "Bank Buk · loan 3310-20 repaid", f"""
TU Opoka paid {sp(F.LOAN_AT_DEATH)} DUK on 12 July 2024: the principal outstanding on 17 May 2024. The loan is repaid in full. The instalment collected on 5 June 2024 ({sp(F.JUNE_INSTALMENT)} DUK) was returned to the account {F.ACCT} on 16 July 2024.
""")
    doc("D-D21", "TU Opoka S.A.", "2024-06-27", "2024-06-29", "TU Opoka · group life policy GZ-7741: benefit", f"""
The insured, Zbigniew Sowa, died on 17 May 2024. The benefit of {sp(F.GROUP_LIFE)} DUK is paid to the named beneficiary, Halina Sowa, on 28 June 2024, to the account {F.ACCT}.
""")
    lines = "\n".join(f"| ISB{s} | {p.isoformat()} | 5 |" for s, p in F.SERIES)
    doc("D-D22", "The Treasury · savings bonds register", "2024-06-26", "2024-07-01", "Register 55-1904-2271 (Zbigniew Sowa): bonds held on 17 May 2024", f"""
| series | bought | bonds |
|---|---|---|
{lines}

325 bonds in all. None was redeemed before 17 May 2024.
""")
    doc("D-D23", "The Treasury · savings bonds register", "2025-06-20", "2025-06-23", "Bonds moved to register 55-2506-0815 (Halina Sowa)", """
On the division of 12 June 2025 the 325 bonds of register 55-1904-2271 are entered in the register of Halina Sowa, 55-2506-0815, as an heir's (TSB-6, TSB-7), with their series' terms. She may ask for any of them to be redeemed.
""")
    for k, red in enumerate(sim["redemptions"]):
        did = f"D-D24{'abcdefgh'[k]}"
        tab = "\n".join(f"| {x['series']} | {x['bonds']} | {sp(x['value'])} | {sp(x['fee'])} | {sp(x['tax'])} | {sp(x['net'])} |" for x in red["lines"])
        doc(did, "The Treasury · savings bonds register", red["paid"].isoformat(), red["paid"].isoformat(),
            f"Early redemption, register 55-2506-0815 (request of {red['request'].isoformat()})", f"""
Requested on {red['request']:%d.%m.%Y}; paid on {red['paid']:%d.%m.%Y} to the account {F.ACCT}.

| series | bonds | value per bond | fee per bond | tax | paid |
|---|---|---|---|---|---|
{tab}

Paid in all: {sp(red['net'])} DUK.
""")
    doc("D-D25", "Halina Sowa, Jakub Sowa and Oskar Wilczek", "2025-06-24", "2025-06-24", "Lease of the flat ul. Ogrodowa 2/11 (extract)", f"""
Landlords: Halina Sowa (13/16) and Jakub Sowa (3/16); Halina Sowa acts for both and collects the rent. Tenant: Oskar Wilczek. From 1 July 2025, for an indefinite period.
Rent: 1 900.00 DUK a month, due by the 10th of the month, to {F.ACCT}. On 1 March of each year from 2026 the rent is multiplied by the yearly CPI average of the year before (`CPI-AVG`, the year before = 100) over 100, to the ort. No deposit.
""")
    doc("D-D26", "Igor Mazur", "2024-03-12", "2024-03-12", "Tenant's notice: ul. Ogrodowa 2/11", """
I give notice on the lease of the flat ul. Ogrodowa 2/11; the lease ends on 31 May 2024. The rent for May is the last. — Igor Mazur
""")
    doc("D-D27", "Housing association WM Ogrodowa 2", "2025-12-09", "2025-12-11", "WM Ogrodowa 2 · the building fee for flat 11", """
The monthly fee for flat 11, collected on the 5th of each month, or the next business day: 420.00 DUK in 2024, 445.00 DUK in 2025, 468.00 DUK in 2026.
""")
    doc("D-D28", "Termo-Kłodzin sp.", "2026-02-16", "2026-02-16", "Invoice FV/2026/0216: ul. Ogrodowa 2/11", f"""
A new gas boiler, fitted on 16 February 2026: {sp(F.BOILER[2])} DUK, paid by transfer on 18 February 2026.
""")
    doc("D-D29", "Domy Lenia sp.", "2025-10-06", "2025-10-08", "Valuation: ul. Ogrodowa 2/11, Kłodzin", """
The flat, let, as of 6 October 2025: 240 000.00 DUK.
""")
    doc("D-D30", "Domy Lenia sp.", "2026-09-15", "2026-09-15", "Offer for a share in ul. Ogrodowa 2/11", f"""
A buyer offers {sp(F.OFFER[1])} DUK for Halina Sowa's 13/16 share in the flat, the deed on 30 November 2026, the price paid that day; the tenant stays. Our fee is 2 % of the price, paid on the deed.
""")
    # R1: the generator writes the world, so it sees every figure
    doc("D-D31", "Revenue Office Kłodzin", "2025-05-19", "2025-05-22", "The 2024 return of Zbigniew Sowa (died 17.05.2024), filed for the estate", f"""
Filed on 14 April 2025 by Halina Sowa for the estate. Tax 0.00 DUK; advances {sp(F.zb_2024_return(known=ALL_KNOWN)['advances'])} DUK; the refund of {sp(F.ZB_REFUND)} DUK is paid on 21 May 2025 to {F.ACCT}.
""")
    doc("D-D32", "Jakub Sowa (e-mail)", "2026-01-14", "2026-01-14", "Kuba's new account", f"""
"Babcia, my old account at Cis is closed, sorry. The new one is {F.KUBA_NEW}. — Kuba"
""")
    doc("D-D33", "Bank Olcha S.A.", "2024-06-20", "2024-06-24", "Bank Olcha · the account after the death of a holder", f"""
Account {F.ACCT}. Balance at the end of 17 May 2024: {sp(c)} DUK. From 20 June 2024 the account is held by Halina Sowa alone.
""")


def main():
    sim = F.sim(known=ALL_KNOWN)  # R1: the generator writes the world, so it sees every figure
    olcha(sim)
    kopia()
    documents(sim)
    print(f"D: {len(sim['rows'])} rows, {len(sim['redemptions'])} redemptions, payouts {sim['division']['child']} / {sim['division']['kuba']}")


if __name__ == "__main__":
    main()
