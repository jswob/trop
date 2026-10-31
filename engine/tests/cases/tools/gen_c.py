#!/usr/bin/env python3
"""Write household C's records and documents (engine/tests/cases/C/records, C/docs) from c_sim, c_facts
and c_tax. The story, the house rules and the questions are written by hand.

Run: python3 -I engine/tests/cases/tools/gen_c.py   (after gen_c_gazette.py)
"""
import pathlib
import sys

HERE = pathlib.Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))

import c_facts as F  # noqa: E402
import c_sim as S  # noqa: E402
import c_tax as X  # noqa: E402
from c_law import D, ZERO, foreign_base  # noqa: E402,F401

C = HERE.parents[0] / "C"
REC, DOC = C / "records", C / "docs"


def pl(x):
    """Bank Jodła: decimal comma, no grouping."""
    return f"{D(x):.2f}".replace(".", ",")


def sp(x):
    """Documents: grouped by spaces, decimal point."""
    return f"{D(x):,.2f}".replace(",", " ")


def dmy(d):
    return f"{d.day:02d}.{d.month:02d}.{d.year}"


def write(path, text):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text, encoding="utf-8")


def doc(n, title, meta, body):
    head = f"# C-D{n} · {title}\n\n- id: C-D{n}\n" + "".join(f"- {k}: {v}\n" for k, v in meta.items())
    write(DOC / f"C-D{n}.md", head + "\n" + body.strip() + "\n")

# --------------------------------------------------------------------------- records


def jodla(name, acc, holder, rows, opening, start, end):
    out = ["Bank Jodła S.A. — historia rachunku", f"Rachunek: {acc}", f"Posiadacz: {holder}",
           f"Okres: {dmy(start)} - {dmy(end)}", f"Saldo początkowe: {pl(opening)}", "",
           "Data operacji;Typ;Kontrahent;Tytuł;Kwota;Saldo po operacji"]
    for r in rows:
        if start <= r["date"] <= end:
            out.append(f"{dmy(r['date'])};{r['type']};{r['cp']};{r['title']};{pl(r['amount'])};{pl(r['balance'])}")
    write(REC / name, "\n".join(out) + "\n")


def lazur_csv(lz):
    rows = [(r["date"], "FLR", r) for r in lz["flr"]] + [(r["date"], "DUK", r) for r in lz["duk"]]
    rows.sort(key=lambda x: (x[0], x[1]))
    out = ["# Bank Lazur S.A. | rachunek wielowalutowy " + F.ACC_LAZUR + " | Jan Wrona",
           "# subkonta: DUK, FLR | wyciąg 2024-06-28..2026-09-30",
           "data,waluta,typ,kontrahent,opis,kwota,saldo"]
    for d, cur, r in rows:
        out.append(f"{d.isoformat()},{cur},{r['type']},\"{r['cp']}\",\"{r['title']}\",{r['amount']:.2f},{r['balance']:.2f}")
    write(REC / "lazur-jan.csv", "\n".join(out) + "\n")


def travel():
    out = ["dzień;nocleg;w Rosvaldzie;praca"]
    for r in F.travel_log():
        out.append(f"{r['date'].isoformat()};{r['night']};{'tak' if r['rosvald'] else 'nie'};{'tak' if r['work'] else 'nie'}")
    write(REC / "dziennik-jana.csv", "\n".join(out) + "\n")


def chat():
    lines = ["[eksport rozmowy: Marta ↔ Jan, wybrane wiadomości o zakupach]"]
    for ts, who, text in [
        ("2025-02-03 18:12", "Marta", "Pralka z Elektro-Domu, 2 899 DUK. Bierzemy?"),
        ("2025-02-04 07:40", "Jan", "Tak."),
        ("2025-04-10 17:05", "Jan", "Kupiłem rower w Ostrel, 600 FLR, z karty florenowej."),
        ("2025-11-27 20:05", "Marta", "Sofa z Kaliny, 3 400. Zgoda?"),
        ("2025-12-05 16:30", "Marta", "Kupiłam sofę, przywiozą w piątek."),
        ("2025-12-10 21:30", "Jan", "Laptop do pracy, 4 199. OK?"),
        ("2025-12-11 06:55", "Marta", "Nie. Poczekaj do stycznia."),
        ("2025-12-20 12:14", "Jan", "Wziąłem laptopa, był w promocji."),
        ("2026-03-02 19:44", "Jan", "Domek na narzędzia z Ogrodu i Domu, 2 650?"),
        ("2026-03-12 22:10", "Marta", "Tak, bierz."),
        ("2026-06-06 11:02", "Marta", "Płytki do łazienki kupione, zapłaciłam dwa razy kartą, bo limit."),
    ]:
        lines.append(f"[{ts}] {who}: {text}")
    write(REC / "zgody.txt", "\n".join(lines) + "\n")


def registers(fm):
    s = ["nr;data wystawienia;nabywca;netto;VAT;brutto;zapłacono"]
    for x in fm["sales"]:
        s.append(f"{x['no']};{x['date'].isoformat()};{x['client']};{pl(x['net'])};{pl(x['vat'])};{pl(x['net'] + x['vat'])};{x['paid'].isoformat()}")
    write(REC / "warsztat-sprzedaz.csv", "\n".join(s) + "\n")
    c = ["nr;data faktury;dostawca;co;netto;VAT;zapłacono"]
    for x in fm["costs"]:
        c.append(f"{x['no']};{x['date'].isoformat()};{x['supplier']};{x['what']};{pl(x['net'])};{pl(x['vat'])};{x['paid'].isoformat()}")
    write(REC / "warsztat-koszty.csv", "\n".join(c) + "\n")

# --------------------------------------------------------------------------- documents


def docs(A, T):
    from fam.home import ALL_KNOWN, run_payer
    m24, m25 = X.marta(2024, known=ALL_KNOWN), X.marta(2025, known=ALL_KNOWN)  # R1: the generator writes the world, so it sees every figure
    _, t24 = run_payer(F.marta_payments(2024), reducing=True, known=ALL_KNOWN)  # R1: the generator writes the world, so it sees every figure
    _, t25 = run_payer(F.marta_payments(2025), reducing=True, known=ALL_KNOWN)  # R1: the generator writes the world, so it sees every figure
    _, tp = run_payer(F.parental_2024(), reducing=False, known=ALL_KNOWN)  # R1: the generator writes the world, so it sees every figure
    doc("01", "Marriage certificate (extract)", {"issuer": "Civil Registry Office, Zabór", "issued": "2016-09-19"},
        f"Jan Wrona (born 1984-02-05) and Marta Sowa (born 1986-07-22) married in Zabór on 2016-09-17. The wife takes the "
        "name Wrona. Annotation: the spouses made no marital contract.")
    doc("02", "Birth certificate (extract)", {"issuer": "Civil Registry Office, Zabór", "issued": "2023-03-14"},
        "Stanisław Wrona, born in Zabór on 2023-03-09; mother Marta Wrona, father Jan Wrona. He lives with both parents "
        "at ul. Graniczna 12, Zabór.")
    doc("03", "SIO decision on parental pay", {"issuer": "SIO, Zabór branch", "issued": "2023-03-27"},
        f"Marta Wrona is granted parental pay for the birth of 2023-03-09, for 52 weeks from that day, at 81.5 % (elected "
        f"2023-03-20). Base: {sp(F.PARENTAL_BASE)} DUK. A day's pay: {sp(F.PARENTAL_DAY)} DUK. The last day of the period is "
        f"{F.PARENTAL_END.isoformat()}. Each month's pay is paid by the 10th of the next month into the account "
        f"{F.ACC_JOINT}.")
    doc("04", "SIO yearly statement 2024 (parental pay)", {"issuer": "SIO, Zabór branch", "issued": "2025-01-24"},
        f"Payer: SIO. Recipient: Marta Wrona. 2024: income {sp(tp['income'])}; contributions 0.00; costs 0.00; base "
        f"{sp(tp['base'])}; advances withheld {sp(tp['advance'])} DUK. Payments: " +
        "; ".join(f"{p['date']} {sp(p['gross'])} (for {p['for']}, {p['days']} days)" for p in F.parental_2024()) + ".")
    doc("05", "Employer's yearly statement 2024", {"issuer": "Przychodnia Pod Lipami sp. z o.o., Zabór", "issued": "2025-01-28"},
        f"Employee: Marta Wrona, nurse. 2024: gross pay {sp(t24['income'])}; social contributions {sp(t24['soc'])}; health "
        f"contribution {sp(t24['health'])}; costs {sp(t24['costs'])}; base {sp(t24['base'])}; advances withheld "
        f"{sp(t24['advance'])} DUK. A reducing statement is on file for 2024.")
    doc("06", "Employer's letters on pay", {"issuer": "Przychodnia Pod Lipami sp. z o.o.", "issued": "2024-02-26; 2024-12-16"},
        "2024-02-26: Marta Wrona returns to work on 2024-03-07 after her parental leave, at 7 350.00 DUK a month gross; "
        "March 2024 is paid for 25 of its 31 days. Pay is paid on the 28th, or the business day before.\n\n"
        "2024-12-16: from January 2025 the gross pay is 7 700.00 DUK a month.")
    doc("07", "Employer's letters on unpaid leave", {"issuer": "Przychodnia Pod Lipami sp. z o.o.", "issued": "2025-07-30; 2026-08-20"},
        "2025-07-30: unpaid leave is granted to Marta Wrona from 2025-09-01 to 2026-08-31. No pay is paid for those months.\n\n"
        "2026-08-20: Marta Wrona returns on 2026-09-01, at 7 900.00 DUK a month gross.")
    doc("08", "Employer's yearly statement 2025", {"issuer": "Przychodnia Pod Lipami sp. z o.o.", "issued": "2026-01-27"},
        f"Employee: Marta Wrona. 2025: gross pay {sp(t25['income'])}; social contributions {sp(t25['soc'])}; health "
        f"contribution {sp(t25['health'])}; costs {sp(t25['costs'])}; base {sp(t25['base'])}; advances withheld "
        f"{sp(t25['advance'])} DUK.")
    doc("09", "Business register: Warsztat Stolarski Jan Wrona", {"issuer": "Business Register, Zabór", "issued": "2024-06-17"},
        "Sole trader: Jan Wrona, joinery, ul. Fabryczna 3, Zabór. Started 2021-02-01. Tax: PIT-10 form (a), no change "
        "notified. VAT: registered taxpayer, monthly returns. SIO: sickness insurance elected from the start.\n\n"
        "Entry of 2024-06-17: activity ended on 2024-06-14.")
    doc("10", "Lease of the workshop (extract)", {"issuer": "Hala Przemysłowa Zabór sp. z o.o.", "issued": "2021-01-20"},
        "Rent 2 600.00 DUK a month net, plus VAT, invoiced on the first business day of the month and paid within 2 days. "
        "Ended by Jan Wrona's notice of 2024-03-28, on 2024-06-30.")
    doc("11", "Bank Jodła: closing of the business account", {"issuer": "Bank Jodła S.A.", "issued": "2024-06-28"},
        f"The account {F.ACC_FIRM} (Warsztat Stolarski Jan Wrona) is closed on 2024-06-28 at the holder's request. Its "
        f"balance, {sp(A['firm']['closing'])} DUK, was transferred the same day to {F.ACC_JOINT}. Payments due after the "
        "closing are to be made from another account.")
    doc("12", "VAT return for June 2024: confirmation", {"issuer": "Revenue Office, Zabór", "issued": "2024-07-15"},
        "Received on 2024-07-15: the VAT return of Jan Wrona for June 2024, marked as the last (activity ended 2024-06-14), "
        f"with a request to refund the excess into {F.ACC_JOINT}.")
    doc("13", "Health contribution settlement 2024: confirmation", {"issuer": "SIO, Zabór branch", "issued": "2025-04-14"},
        f"Received on 2025-04-14: Jan Wrona's settlement of the health contribution for 2024 (PIT-10 form (a), activity "
        f"2024-01-01 to 2024-06-14), with a request to refund any overpayment into {F.ACC_JOINT}.")
    doc("14", "Employment contract (extract)", {"issuer": "Montagewerk Ostrel AG, Ostrel (Rosvald)", "issued": "2024-06-10"},
        "Employee: Jan Wrona, of Zabór (Lenia). Assembly fitter, from 2024-07-01, at Ostrel, without end date.\n\n"
        "1. Base pay 4 800.00 FLR a month; overtime 37.50 FLR an hour, in whole hours.\n"
        "2. Pay for a month is paid on the 5th of the next month, or the Rosvald business day before, into the "
        f"employee's account {F.ACC_LAZUR} (FLR).\n3. The pay terms are reviewed each year, with effect from the pay for April.\n"
        "4. Work away from Ostrel: travel and lodging are paid by the employer.")
    doc("15", "Class notice 2024/25", {"issuer": "Jan Wrona, to Montagewerk Ostrel AG", "issued": "2024-07-01",
                                        "received": "2024-07-01 (employer's stamp)"},
        "For the tax year 2024/25: married, living with a child under 16 (Stanisław, born 2023-03-09). Class 2 requested.")
    doc("16", "Annexes to the employment contract", {"issuer": "Montagewerk Ostrel AG", "issued": "2025-03-14; 2026-03-13"},
        "2025-03-14: from the pay for April 2025, base pay 5 000.00 FLR and overtime 39.00 FLR an hour.\n\n"
        "2026-03-13: from the pay for April 2026, base pay 5 150.00 FLR and overtime 40.25 FLR an hour.")
    doc("17", "Class notice 2025/26", {"issuer": "Jan Wrona, to Montagewerk Ostrel AG", "issued": "2025-05-19",
                                        "received": "2025-05-20 (employer's stamp)"},
        "For the tax year 2025/26: married, living with a child under 16. Class 2 requested.")
    doc("18", "Class notice 2026/27", {"issuer": "Jan Wrona, to Montagewerk Ostrel AG", "issued": "2026-03-31",
                                        "received": "2026-04-01 (employer's stamp)"},
        "For the tax year 2026/27: married, living with a child under 16. Class 2 requested.")
    a = A["lazur"]["a2425"]
    doc("19", "RTB decision: yearly assessment 2024/25", {"issuer": "Rosvald Tax Board, Ostrel office", "issued": "2025-09-10"},
        f"Claim of Jan Wrona filed 2025-06-12. Days of presence in 2024/25: at least 183, as his log shows. Pay "
        f"{sp(a['gross'])} FLR; contributions {sp(a['soc'])}; allowance (class 2, 12 months) {sp(a['allowance'])}; taxable "
        f"{sp(a['taxable'])}; yearly tax {sp(a['tax'])}; withheld {sp(a['withheld'])}. Refund: {sp(a['refund'])} FLR, "
        f"paid within 14 days to {F.ACC_LAZUR}.")
    slip_lines = ["| pay for | paid | class | hours | gross | RSI | tax | net |", "|---|---|---|---|---|---|---|---|"]
    for s in A["lazur"]["slips"]:
        if s["work"] == F.MISSING_SLIP:
            continue
        slip_lines.append(f"| {s['work']} | {s['paid'].isoformat()} | {s['class']} | {s['hours']} | {sp(s['gross'])} | "
                          f"{sp(s['soc'])} | {sp(s['tax'])} | {sp(s['net'])} |")
    doc("20", "Payslips, July 2024 – August 2026", {"issuer": "Montagewerk Ostrel AG", "issued": "monthly"},
        "Jan's payslips as kept, one line per slip (FLR). The slip for October 2025 is missing.\n\n" + "\n".join(slip_lines))
    doc("21", "RTB: claim received, 2025/26", {"issuer": "Rosvald Tax Board, Ostrel office", "issued": "2026-07-16"},
        "Received on 2026-07-14: Jan Wrona's claim of a yearly assessment for 2025/26. A decision will follow.")
    doc("22", "RFF decision on the child benefit", {"issuer": "Rosvald Family Fund", "issued": "2024-11-20"},
        "Claim of Jan Wrona filed 2024-11-04 for Stanisław Wrona (born 2023-03-09), who lives in Lenia. For July to November 2024 "
        "the RFF pays the full benefit, as the agreement's Part II has no effect before December 2024. From December 2024, "
        "while the mother works in Lenia, Lenia pays first (T-12 .1 (a)) and the RFF pays the difference each month. The months "
        "July–September 2024 are paid within 30 days; then each quarter by the last business day of the month after it, "
        f"to {F.ACC_LAZUR} (FLR).")
    doc("23", "RFF letter after the report of 2025-11-24", {"issuer": "Rosvald Family Fund", "issued": "2025-12-05"},
        "Jan Wrona reported on 2025-11-24 that the mother has been on unpaid leave since 2025-09-01. From September 2025 "
        "the RFF pays first, in full (T-12 .1 (b), T-13 .1). The difference for September 2025, already paid as a "
        "difference in October, is made up by 2025-12-19.")
    doc("24", "FBO decisions on the child benefit", {"issuer": "FBO, Zabór", "issued": "2024-05-14; 2025-05-13"},
        f"Marta Wrona, for Stanisław Wrona: 650.00 DUK a month for the benefit period 2024-06-01 – 2025-05-31, and again "
        f"for 2025-06-01 – 2026-05-31, paid for each month by its last day into {F.ACC_JOINT}. Any change in work or in "
        "another state's benefit must be reported (FAM-2 .1).")
    doc("25", "FBO decision: benefit paid without entitlement", {"issuer": "FBO, Zabór", "issued": "2025-12-08"},
        "On the report of 2025-11-24: since 2025-09-01 neither parent works in Lenia and the father works in Rosvald, so "
        "Rosvald pays first; Rosvald's benefit is higher than Lenia's, so no difference is due from Lenia. The benefit for "
        "September, October and November 2025, 1 950.00 DUK, was paid without entitlement and is to be repaid in 6 "
        "instalments of 325.00 DUK, by the 15th of each month from January to June 2026. Payments stop from December 2025.")
    doc("26", "FBO decision: benefit resumed", {"issuer": "FBO, Zabór", "issued": "2026-09-15"},
        "On the report of 2026-09-02: the mother works in Lenia again from 2026-09-01. From September 2026 Lenia pays "
        "first: 650.00 DUK a month, for September 2026 by 2026-09-30.")
    _, inst = F.mortgage([])
    doc("27", "Mortgage contract (extract)", {"issuer": "Bank Jodła S.A.", "issued": "2019-05-15"},
        f"Borrowers: Jan Wrona and Marta Wrona. Loan {F.MORTGAGE}: 400 000.00 DUK, paid out 2019-05-20, for the house "
        "ul. Graniczna 12, Zabór. Fixed rate 6.20 % a year for the whole term.\n\n"
        f"1. 300 monthly instalments of {sp(inst)} DUK from 2019-06-15, on the 15th of each month, or the next business "
        "day where the 15th isn't one, debited from the joint account.\n"
        "2. Each instalment's interest is 1/12 of 6.20 % of the principal owed at the end of the day before it, to the ort, "
        "half up; the rest of the instalment repays principal. The last instalment is what is then owed.\n"
        "3. Overpayments may be made on any day, without fee; they reduce the principal on their day and shorten the term; "
        "the instalment stays the same.")
    yr = [r for r in A["mortgage"] if r["date"] <= S.date(2025, 12, 31)]
    doc("28", "Mortgage: yearly statement 2025", {"issuer": "Bank Jodła S.A.", "issued": "2026-01-09"},
        f"Loan {F.MORTGAGE}. Principal owed on 2025-12-31: {sp(yr[-1]['balance'])} DUK. Overpayments in 2025: " +
        "; ".join(f"{r['date'].isoformat()} {sp(r['amount'])}" for r in yr if r["kind"] == "overpayment" and r["date"].year == 2025) + ".")
    doc("29", "Car sale contract", {"issuer": "Auto-Komis Rybak, Zabór", "issued": "2024-07-02"},
        f"Seller: Auto-Komis Rybak. Buyer: Jan Wrona. A used passenger car, first registered 2019, price 39 500.00 DUK, paid "
        f"today by transfer from {F.ACC_LAZUR} (DUK).")
    doc("30", "Notary's notice of payment from the deposit", {"issuer": "Notary Ewa Brzoza, Kłodzin", "issued": "2025-06-16"},
        "Estate of Zbigniew Sowa, agreement of 2025-06-12. To Marta Wrona (née Sowa), heir of 3/16: her payout of "
        f"{sp(S.INHERIT[1])} DUK was paid today from the deposit to her account {F.ACC_MARTA}.")
    doc("31", "Loan agreement", {"issuer": "Marta Wrona and Jan Wrona", "issued": "2025-06-20"},
        "Marta Wrona lends to the spouses' community 30 000.00 DUK from her own money (her inheritance from her father), "
        "paid today into the joint account, for an overpayment of the mortgage. The terms are our house rule C-HR7, "
        "which this agreement signs. Signed: Marta Wrona, Jan Wrona.")
    doc("32", "Fundacja Pod Dębem: receipts; the RO's list", {"issuer": "Fundacja Pod Dębem, Zabór; Revenue Office",
                                                            "issued": "2024-12-17; 2025-12-16"},
        f"2024-12-17: received from Marta and Jan Wrona, by transfer from {F.ACC_JOINT} on 2024-12-16, 21 300.00 DUK, a gift.\n\n"
        f"2025-12-16: received from Marta and Jan Wrona, by transfer from {F.ACC_JOINT} on 2025-12-15, 5 200.00 DUK, a gift.\n\n"
        "Extract of the RO's list of public-benefit bodies for 2024 and 2025: Fundacja Pod Dębem, Zabór — listed.")
    doc("33", "Invoice: tiles", {"issuer": "Płytki Zabór", "issued": "2026-06-06"},
        "Invoice FV/2026/0612. Buyer: Marta Wrona, ul. Graniczna 12, Zabór. Bathroom tiles, one lot, 2 800.00 DUK "
        "(VAT included). Paid by card in two payments of 1 400.00.")
    doc("34", "RO confirmations of returns", {"issuer": "Revenue Office, Zabór", "issued": "2025-04-28; 2025-10-20; 2026-04-27"},
        "2025-04-28: received the joint return of Marta and Jan Wrona for 2024.\n\n"
        "2025-10-20: received their corrected joint return for 2024 (filed after the Rosvald refund of 2025-09-24).\n\n"
        "2026-04-27: received their joint return for 2025. In it, Jan Wrona's Rosvald pay paid from 2025-01-03 on is "
        "stated as exempt under the treaty, with progression.")
    doc("35", "Bank Jodła: Marta's own account, history", {"issuer": "Bank Jodła S.A.", "issued": "2024-01-05"},
        f"Account {F.ACC_MARTA}, holder Marta Wrona (née Sowa), opened 2014-04-02. Balance at the end of 2016-09-16: "
        "45 245.63 DUK. From 2016-09-17 to 2023-12-31 the account received only interest (net of tax): 2 104.37 DUK in all; "
        "nothing was paid out. Balance at the end of 2023-12-31: 47 350.00 DUK. Interest: 1.20 % a year on each day's "
        "balance, credited at each quarter's end, tax withheld.")
    doc("36", "Irena's letter", {"issuer": "Irena Wrona", "issued": "2023-04-01"},
        f"From April I'll send 300 every month, on the 3rd, to your joint account {F.ACC_JOINT}, titled 'dla Stasia'. "
        "It's Staś's money, not yours: keep it for him.")

# --------------------------------------------------------------------------- run


def main():
    from fam.home import ALL_KNOWN
    taxes, T = X.household_taxes(known=ALL_KNOWN)  # R1: the generator writes the world, so it sees every figure
    A = S.accounts(taxes, known=ALL_KNOWN)  # R1: the generator writes the world, so it sees every figure
    jr, pr, mr = A["joint"].run(), A["pot"].run(), A["marta"].run()
    jodla("jodla-wspolny.csv", F.ACC_JOINT, "MARTA WRONA, JAN WRONA", jr, S.OPEN["joint"], S.T0, S.T1)
    jodla("jodla-skarbonka.csv", F.ACC_POT + " (Skarbonka)", "MARTA WRONA, JAN WRONA", pr, S.OPEN["pot"], S.T0, S.T1)
    jodla("jodla-marta.csv", F.ACC_MARTA, "MARTA WRONA", mr, S.OPEN["marta"], S.T0, S.T1)
    jodla("jodla-warsztat.csv", F.ACC_FIRM, "WARSZTAT STOLARSKI JAN WRONA", A["firm"]["rows"], S.OPEN["firm"], S.T0, S.date(2024, 6, 28))
    lazur_csv(A["lazur"])
    travel()
    chat()
    registers(A["firm"])
    docs(A, T)


if __name__ == "__main__":
    main()
