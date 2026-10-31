#!/usr/bin/env python3
"""Write household A's records and documents from fam/a.py's facts and simulation.

Run: python3 -I tmp/cases/tools/gen_a.py   (story.md, house-rules.md and questions.md are written by hand)
"""
import pathlib
import sys
from datetime import date, timedelta
from decimal import Decimal

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))

from fam import a  # noqa: E402
from fam.home import ALL_KNOWN, CAL, ZERO, D, duk, ort, figure, price_on, units_down  # noqa: E402

ROOT = pathlib.Path(__file__).resolve().parents[1]
OUT = ROOT / "A"


def c2(x):
    """Decimal comma, two places."""
    return f"{D(x):.2f}".replace(".", ",")


def c4(x):
    return f"{D(x):.4f}".replace(".", ",")


def sp(x):
    """The issuer's print: 12 345.67"""
    s = f"{D(x):,.2f}".replace(",", " ")
    return s


def dmy(d, sep="."):
    return d.strftime(f"%d{sep}%m{sep}%Y")


def write(rel, text):
    p = OUT / rel
    p.parent.mkdir(parents=True, exist_ok=True)
    p.write_text(text, encoding="utf-8")


def doc(did, issuer, issued, received, title, body):
    head = f"# {did} · {title}\n\n- id: {did}\n- issuer: {issuer}\n- issued: {issued}\n- received: {received}\n\n"
    write(f"docs/{did}.md", head + body.strip() + "\n")


# --------------------------------------------------------------------------- records

def joint_statements(sim):
    rows = sim["joint"]
    y, m = 2024, 8
    while (y, m) <= (2025, 12):
        mrows = [r for r in rows if r["date"].year == y and r["date"].month == m]
        before = [r for r in rows if r["date"] < date(y, m, 1)]
        opening = before[-1]["balance"] if before else a.JOINT_OPEN[1]
        last = date(y, m, 28)
        while (last + timedelta(days=1)).month == m:
            last += timedelta(days=1)
        out = ["Bank Wrzos SA;Account history", "Account;LN27 1140 2004 0000 3402 7716 0412",
               "Holders;ANNA LIS, TOMASZ LIS", f"Period;{dmy(date(y, m, 1))};{dmy(last)}", f"Opening balance;{c2(opening)}",
               "Booking date;Type;Counterparty;Title;Amount;Balance after"]
        for r in mrows:
            out.append(f"{dmy(r['date'])};{r['type']};{r['counterparty']};{r['title']};{c2(r['amount'])};{c2(r['balance'])}")
        write(f"records/wrz-joint-{y}-{m:02d}.csv", "\n".join(out) + "\n")
        y, m = a.next_month(y, m)


def wrapped(book):
    out = ["account,date,type,fund,units,price_DUK,amount_DUK"]
    for r in sorted(book.rows["WRAP"], key=lambda r: r["date"]):
        u = "" if r["units"] == "" else f"{r['units']:.4f}"
        px = "" if r["price"] == "" else f"{r['price']:.2f}"
        out.append(f"BRZ-WRAP-0091-A,{r['date']},{r['type']},{r['fund']},{u},{px},{r['amount']:.2f}")
    write("records/brz-wrapped-ania.csv", "\n".join(out) + "\n")


RZK_CASH = {"DEPOSIT": 1, "BUY": -1, "SELL": 1, "WITHDRAWAL": 1, "DISTRIBUTION": 1, "TAX WITHHELD": 1, "TRANSFER IN": 0}


def rzeka(book, acct, client, name):
    out = [f"Rzeka Securities - transaction history - client {client} ({name})", "Date;Operation;Fund;Units;Price;Value;Fee;Cash after;Note"]
    cash = ZERO  # cash after, in date order: a sale's withdrawal is booked when it is sold, dated days later
    for r in sorted(book.rows.get(acct, []), key=lambda r: r["date"]):
        cash += RZK_CASH[r["op"]] * r["value"] - (r["fee"] if RZK_CASH[r["op"]] else ZERO)
        r = dict(r, cash=cash)
        u = "" if r.get("units", "") == "" else c4(r["units"])
        px = "" if r.get("price", "") == "" else (c4(r["price"]) if r["op"] == "DISTRIBUTION" else c2(r["price"]))
        out.append(f"{dmy(r['date'], '/')};{r['op']};{r.get('fund', '')};{u};{px};{c2(r['value'])};{c2(r['fee'])};{c2(r['cash'])};{r.get('note', '')}")
    assert cash == book.cash.get(acct, ZERO), (acct, cash)
    write(f"records/rzk-{client}-{name.split()[0].lower()}.csv", "\n".join(out) + "\n")


def kopia(book):
    out = ["DM Kopia\tsecurities account 30-118-552 (ANNA LIS)\topened 12.09.2023\tclosed 29.02.2024", "Date\tEvent\tInstrument\tQty\tNote"]
    for r in book.rows["KOP-A"]:
        out.append(f"{r['date'].isoformat()}\t{r['op']}\t{r['fund']}\t{r['units']}\t{r['note']}")
    write("records/kop-30-118-552-ania.csv", "\n".join(out) + "\n")


def wcp(sim, who, name, employer):
    out = [f"Wektor Workplace Plans - participant register - {name} - plan of {employer}", "Date;Source;Amount (DUK);NAV (DUK);Units;Units after",
           f"01.01.2021;opening balance;;;;{c4(a.WCP_OPEN[who])}"]
    for r in sim["wcp"][who]:
        out.append(f"{dmy(r['date'])};{r['source']};{c2(r['amount'])};{c4(r['price'])};{c4(r['units'])};{c4(r['after'])}")
    write(f"records/wek-wcp-{name.split()[1].lower()}-{name.split()[0].lower()}.csv", "\n".join(out) + "\n")


LAZ_RATES = [(date(2021, 3, 1), D("0.031")), (date(2024, 1, 1), D("0.042")), (date(2025, 7, 1), D("0.036"))]  # the last: not in the dossier
LAZ_DEPOSITS = [(date(2021, 3, 1), D("10000.00")), (date(2022, 3, 1), D("5000.00")), (date(2023, 2, 1), D("5000.00"))] + a.OWN_TO_LAZUR


def lazur():
    """Bank Lazur savings (A-D31): interest on the daily balance at the yearly rate / 365, credited each quarter's last day."""
    rows, bal, acc = [], ZERO, D(0)
    d = date(2021, 3, 1)
    while d <= date(2025, 12, 31):
        for dd, amt in LAZ_DEPOSITS:
            if dd == d:
                bal += amt
                rows.append((d, "deposit", amt, bal))
        rate = [r for f, r in LAZ_RATES if f <= d][-1]
        acc += bal * rate / 365
        nd = d + timedelta(days=1)
        if nd.month in (1, 4, 7, 10) and nd.day == 1:
            gross = ort(acc)
            tax = duk(gross * D("0.19"))
            bal += gross
            rows.append((d, "interest", gross, bal))
            bal -= tax
            rows.append((d, "tax withheld", -tax, bal))
            acc = D(0)
        d = nd
    return rows


def lazur_file(rows):
    out = ["date,description,amount,balance"]
    for d, desc, amt, bal in rows:
        if d <= date(2025, 6, 30):
            out.append(f"{d},{desc},{amt:.2f},{bal:.2f}")
    write("records/laz-savings-1020-0044-7310.csv", "\n".join(out) + "\n")


def ania_savings():
    rows, bal = [], ZERO
    d = date(2025, 6, 2)
    acc = D(0)
    while d <= date(2025, 12, 31):
        if d == a.ESTATE_PAYOUT[0]:
            bal += a.ESTATE_PAYOUT[1]
            rows.append({"date": d, "type": "INCOMING TRANSFER", "counterparty": "Kancelaria Notarialna Brzoza – escrow",
                         "title": "payout, division of the estate of Zbigniew Sowa, A. Lis", "amount": a.ESTATE_PAYOUT[1], "balance": bal})
        acc += bal * D("0.024") / 365
        nd = d + timedelta(days=1)
        if nd.day == 1:
            gross = ort(acc)
            tax = duk(gross * D("0.19"))
            bal += gross
            rows.append({"date": d, "type": "INTEREST", "counterparty": "Bank Wrzos SA", "title": "interest", "amount": gross, "balance": bal})
            bal -= tax
            rows.append({"date": d, "type": "TAX", "counterparty": "Bank Wrzos SA", "title": "tax on interest 19%", "amount": -tax, "balance": bal})
            acc = D(0)
        d = nd
    out = ["Bank Wrzos SA;Account history", "Account;LN61 1140 2004 0000 3402 9921 5503", "Holders;ANNA LIS",
           f"Period;02.06.2025;31.12.2025", "Opening balance;0,00", "Booking date;Type;Counterparty;Title;Amount;Balance after"]
    for r in rows:
        out.append(f"{dmy(r['date'])};{r['type']};{r['counterparty']};{r['title']};{c2(r['amount'])};{c2(r['balance'])}")
    write("records/wrz-ania-savings-2025.csv", "\n".join(out) + "\n")
    return rows


# --------------------------------------------------------------------------- documents

def yearly_statements():
    names = {"NURT": ("Nurt Software sp.", "TOMASZ LIS"), "KALINA": ("Kalina Logistyka sp.", "ANNA LIS"),
             "SIO": ("Social Insurance Office, branch Dębowo", "ANNA LIS")}
    for year in (2021, 2022, 2023):
        pr = a.payroll(year, known=ALL_KNOWN)  # R1: the generator writes the payslips as paid, seeing every figure
        for payer, (rows, tot) in pr.items():
            issuer, who = names[payer]
            did = f"A-D13-{payer}-{year}"
            hd = f"| health contributions deductible from tax (7.75 %) | {sp(tot['health_ded'])} |\n" if year <= 2021 else ""
            body = f"""Yearly information on income and advances, tax year {year}, for {who}.

| | DUK |
|---|---|
| income (gross pay{', plan contributions paid by the employer' if payer != 'SIO' else ', parental pay'}) | {sp(tot['income'])} |
| costs | {sp(tot['costs'])} |
| social contributions deducted | {sp(tot['soc'])} |
| health contributions | {sp(tot['health'])} |
{hd}| advances withheld | {sp(tot['advance'])} |
| reducing statement on file | {'yes' if payer != 'SIO' else 'no'} |
"""
            issued = date(year + 1, 2, 20) if payer != "SIO" else date(year + 1, 2, 26)
            doc(did, issuer, issued, issued + timedelta(days=4), f"{issuer.split(',')[0]} · yearly information {year}", body)


def documents(sim):
    car = sim["car"]
    par = a.parental()
    doc("A-D01", "Nurt Software sp.", "2019-03-01", "2019-03-01", "Nurt Software · employment contract (extract)", """
Employer: Nurt Software sp., Dębowo. Employee: Tomasz Lis, born 2 July 1989.

§4. Pay is monthly. The pay for a month is paid on the 10th of the next month; where the 10th is not a business day, on the last business day before it.

§5. The amount of the monthly gross pay is set in writing each year.

Annex: the employee has filed with the employer the statement that lets the employer take 1/12 of the yearly reducing amount off each advance (PIT-4). It stays on file until withdrawn.
""")
    for y in range(2021, 2026):
        doc(f"A-D02-{y}", "Nurt Software sp.", f"{y - 1}-12-15", f"{y - 1}-12-15", f"Nurt Software · pay from January {y}",
            f"Dear Tomasz, from the pay for January {y} your monthly gross pay is {sp(a.T_GROSS[y])} DUK.")
    doc("A-D03", "Nurt Software sp.", "2022-10-03", "2022-10-03", "Nurt Software · the acquisition and your options", f"""
Nurt Software has been acquired by Arka Systems. Your vested options are cashed out: a one-off gross amount of {sp(a.T_CASHOUT[1])} DUK, paid through payroll with your pay for October 2022, on 10 November 2022. Payroll treats it as pay: contributions, the workplace plan and the advance apply as to any pay. Your employment continues unchanged.
""")
    doc("A-D04", "Nurt Software sp.", "2024-11-20", "2024-11-20", "Nurt Software · annex: payday", """
From the pay for December 2024, paid in January 2025, §4 reads: the pay for a month is paid on the 15th of the next month; where the 15th is not a business day, on the last business day before it. The workplace-plan contributions move with it.
""")
    doc("A-D05", "Kalina Logistyka sp.", "2018-06-01", "2018-06-01", "Kalina Logistyka · employment contract (extract)", """
Employer: Kalina Logistyka sp., Dębowo. Employee: Anna Lis, born 14 March 1991.

§3. Pay is monthly and is paid on the last business day of the month it is for.

§7. A month not worked in full is paid for the days of the month the employment lasted, at 1/30 of the monthly pay a day.

Annex: the employee has filed the reducing statement (PIT-4) with the employer.
""")
    for y in range(2021, 2026):
        doc(f"A-D06-{y}", "Kalina Logistyka sp.", f"{y - 1}-12-18", f"{y - 1}-12-18", f"Kalina Logistyka · pay from January {y}",
            f"Dear Anna, from January {y} your monthly gross pay is {sp(a.A_GROSS[y])} DUK.")
    doc("A-D06-2023b", "Kalina Logistyka sp.", "2023-04-28", "2023-04-28", "Kalina Logistyka · April 2023 pay", """
Your pay for April 2023 covers 1–26 April (26 days, §7): 4 680.00 DUK gross. From 27 April you are on parental leave; no pay is due from us while it lasts.
""")
    doc("A-D06-2024b", "Kalina Logistyka sp.", "2024-04-25", "2024-04-25", "Kalina Logistyka · return to work", """
Welcome back. From 25 April 2024 your monthly gross pay is 5 700.00 DUK. Your pay for April 2024 covers 25–30 April (6 days, §7): 1 140.00 DUK gross.
""")
    doc("A-D08", "Wektor Workplace Plans", "2019-06-20", "2019-06-20", "Wektor · workplace plan of Nurt Software, terms", """
1. Contributions: the participant's basic 2 % and, where declared, an additional contribution; the employer's basic 1.5 % and the plan's top-up of 1.0 %, each of the month's gross pay.
2. Tomasz Lis has declared an additional contribution of 1 %.
3. The employer pays both parties' contributions for a pay to the plan on the payday of the next pay.
4. Units are bought at the fund's price (Wektor Workplace Plan 2055) of the day the money is credited, to 4 places, rounded down.
""")
    doc("A-D09", "Wektor Workplace Plans", "2019-06-24", "2019-06-24", "Wektor · workplace plan of Kalina Logistyka, terms", """
1. Contributions: the participant's basic 2 %; the employer's basic 1.5 %; no top-up and no additional contribution declared.
2. The employer pays the contributions for a month's pay on the 15th of the next month; where the 15th is not a business day, on the last business day before it.
3. Units are bought at the fund's price of the day the money is credited, to 4 places, rounded down.
""")
    doc("A-D11", "Brzask TFI", "2020-12-10", "2020-12-10", "Brzask · wrapped account, terms and instruction", """
Account BRZ-WRAP-0091-A, holder Anna Lis. A wrapped account under PIT-9.

1. Each contribution is invested on the day it is credited: half in Brzask Global Equity (BRZ-GLB-A), half in Brzask Treasury Bond (BRZ-BND-A), at that day's price; units to 4 places, rounded down; what isn't invested stays on the account as cash.
2. A withdrawal is paid by redeeming units at the day's price, two business days after the request.
""")
    doc("A-D12", "Rzeka Securities", "2020-12-01", "2020-12-01", "Rzeka Securities · terms for fund accounts", """
1. A purchase is placed on the day the money arrives, at that day's price. Fee: 0.25 % of the money placed, at least 5.00 DUK, rounded to the ort; units to 4 places, rounded down; what isn't invested stays as cash.
2. Redemptions are free and paid into the account's cash at the day's price.
3. Distributions are paid into the account's cash, less the 19 % tax withheld.
4. Units transferred in from another broker are booked with the cost basis the transferring broker reports.
5. In February we send each client a yearly tax statement of the sales of the past year.
""")
    yearly_statements()
    doc("A-D15", "Social Insurance Office, branch Dębowo", "2023-05-08", "2023-05-11", "SIO · decision on parental pay", f"""
Anna Lis is granted parental pay for the birth of Zofia Lis on 27 April 2023, for 52 weeks: 27 April 2023 – {par['last']:%d %B %Y}.

- base: the average monthly gross pay of April 2022 – March 2023, {sp(par['average'])} DUK, less 13.45 %: {sp(par['base'])} DUK;
- rate: 81.5 % for the whole period, as elected on 5 May 2023;
- a day's pay: {sp(par['day_pay'])} DUK.

Each month's pay is paid by the 10th of the next month.
""")
    doc("A-D16", "Anna Lis", "2023-07-27", "2023-07-27", "Child benefit claims (copies) with the post office's receipt", """
Two claims for the child benefit for Zofia Lis, born 27 April 2023, signed by Anna Lis on 27 July 2023:
1. for the benefit period June 2022 – May 2023;
2. for the benefit period June 2023 – May 2024.

Registered-post receipt: posted 27 July 2023, 17:42, post office Dębowo 3.
Stamp of the Family Benefits Office on the copy returned with its acknowledgement: "received 31.07.2023".
""")
    doc("A-D17", "Family Benefits Office, Dębowo", "2023-09-12", "2023-09-14", "FBO · decision on the child benefit", """
The child benefit for Zofia Lis is granted to Anna Lis:
- for the period June 2022 – May 2023, from April 2023 (the claim was filed within 3 months of the birth: posted on 27 July 2023);
- for the period June 2023 – May 2024, from July 2023 (the month the claim was filed).
Amounts due for months already past are paid on 25 September 2023.
""")
    px, pd_ = price_on("BRZ-GLB-A", a.GIFT_DAY, ALL_KNOWN)  # R1: the generator writes the world, so it sees every price
    doc("A-D18", "Halina Sowa and Anna Lis", "2023-09-12", "2023-09-12", "Gift contract: fund units", f"""
Halina Sowa, mother, gives Anna Lis, daughter, for Anna alone, 200 units of Brzask Global Equity (BRZ-GLB-A), held at DM Kopia. Halina bought them in three purchases, whose days she didn't keep:

| purchase | units |
|---|---|
| November 2018 | 90 |
| March 2019 | 60 |
| August 2019 | 50 |

The units are transferred today to Anna's account 30-118-552 at DM Kopia.

Signed at DM Kopia, Dębowo, 12 September 2023.
""")
    doc("A-D19", "Anna Lis", "2021-04-30", "2021-04-30", "A note kept with the family papers", """
"30 April 2021. Mum gave me 15 000 DUK for my 30th birthday, in cash, in an envelope. — Ania"
""")
    doc("A-D20", "Anna Lis", "2024-03-14", "2024-03-14", "Report of an acquisition to the Revenue Office (copy)", """
Anna Lis reports the acquisition, by gift from her mother Halina Sowa on 12 September 2023, of 200 units of Brzask Global Equity. Filed at the Revenue Office Dębowo on 14 March 2024 (stamp on the copy).
""")
    rows = [r for r in sim["book"].rows["RZK-A"] if r["op"] == "TRANSFER IN"]
    lines = "\n".join(f"| {r['note'].split('acquired ')[1]} | {r['units']} | {sp(r['value'])} |" for r in rows)
    doc("A-D21", "Rzeka Securities", "2024-02-12", "2024-02-13", "Rzeka · transfer-in confirmation, client 4471-07", f"""
Received from DM Kopia, account 30-118-552, into client 4471-07 (Anna Lis): 200 units of BRZ-GLB-A, with the cost basis DM Kopia reported:

| acquired | units | cost basis (DUK) |
|---|---|---|
{lines}
""")
    st = a.broker_statement_2024(sim["book"])
    doc("A-D22", "Rzeka Securities", "2025-02-20", "2025-02-21", "Rzeka · yearly tax statement 2024, client 4471-07", f"""
Anna Lis, sales of fund units in 2024.

| | DUK |
|---|---|
| proceeds | {sp(st['proceeds'])} |
| cost | {sp(st['cost'])} |
| gain | {sp(st['gain'])} |
| tax at 19 % on the gain, rounded | {sp(st['tax'])} |

To be stated in your return. This statement is prepared from the data in our books.
""")
    doc("A-D23", "Auto Dąb sp.", "2021-05-14", "2021-05-14", "Invoice: a car", """
Tarpan Kombi 1.8, first registered 2019, engine 1 798 cm³, registration WD 4471K. Buyers: Anna Lis and Tomasz Lis. Price 64 000.00 DUK: 19 000.00 paid by transfer on 12 May 2021, 45 000.00 by Bank Wrzos's credit 77-0412.
""")
    doc("A-D24", "Bank Wrzos SA", "2021-05-14", "2021-05-14", "Consumer credit agreement 77-0412 (extract)", f"""
Borrowers: Anna Lis and Tomasz Lis.

1. Amount: 45 000.00 DUK, paid to Auto Dąb on 14 May 2021.
2. Term: 48 months; instalments on the 14th of each month, the first on 14 June 2021, the last on 14 May 2025.
3. Interest: fixed, 7.9 % a year; a month's interest is the balance times 7.9 % / 12, rounded to the ort.
4. Instalments are equal, {sp(car['instalment'])} DUK; the last one settles what is left.
5. Commission: 2 700.00 DUK for the credit's 48 instalment periods, paid by the borrowers on signing.
6. On repayment between instalment days, interest for the days since the last instalment day: the balance times 7.9 % times the days / 365, rounded to the ort.
7. Early repayment: allowed at any time, in full or in part (CC-1).
8. Early repayment fee: 1 % of the amount repaid early where more than a year is left to the last instalment day, 0.5 % otherwise; never more than the law allows (CC-2).
""")
    doc("A-D25", "Orlik Insurance", "2023-08-20", "2023-08-22", "Orlik · compulsory liability policy OC/2023/55102", f"""
Car WD 4471K. Holder: Tomasz Lis. Period: 1 September 2023 – 31 August 2024. Premium: {sp(a.OC['premium'])} DUK, paid.

Terms §9: when the policy ends before its period, the insurer refunds the premium times the days from the day after the end to the period's last day, over the days of the period, rounded down to the ort, less a handling fee of 20.00 DUK.
""")
    doc("A-D26", "Orlik Insurance", "2024-07-25", "2024-07-27", "Orlik · renewal offer", f"""
Your policy for WD 4471K renews on 1 September 2024 for one year at a premium of {sp(a.OC['renewal_premium'])} DUK, payable on 1 September 2024.
""")
    doc("A-D27", "Tomasz Lis, Anna Lis and Marek Dzięcioł", "2024-08-20", "2024-08-20", "Car sale contract", """
The sellers Anna and Tomasz Lis sell Tarpan Kombi WD 4471K to Marek Dzięcioł for 41 500.00 DUK, paid by transfer today. The car is handed over today, 20 August 2024.
""")
    doc("A-D28", "Bank Wrzos SA", "2024-08-21", "2024-08-21", "Bank Wrzos · early repayment statement, credit 77-0412", f"""
To repay the credit in full on 22 August 2024:

| | DUK |
|---|---|
| principal after the instalment of 14 August 2024 | {sp(car['balance'])} |
| interest 15–22 August 2024 ({car['days']} days) | {sp(car['interest'])} |
| early repayment fee | {sp(car['fee'])} |
| to pay | {sp(car['payoff'])} |

The settlement of the credit's cost (CC-1.2) follows within 14 days of the repayment.
""")
    doc("A-D29", "Garaże Kos", "2021-06-01", "2021-06-01", "Garage space contract", """
Space 14, 180.00 DUK a month, paid on the 10th for that month, by standing order. Either side may end it on any day; a month already paid is not refunded.
""")
    doc("A-D30", "Jerzy Grab", "2019-08-20", "2019-08-20", "Lease of the flat, ul. Klonowa 8/3, Dębowo (extract)", """
Rent 3 200.00 DUK a month, due on the 1st. Deposit 6 400.00 DUK, paid on signing, returned when the lease ends, less any damage.
Annex of 2 December 2024: from January 2025 the rent is 3 350.00 DUK.
""")
    doc("A-D31", "Bank Lazur SA", "2021-03-01", "2021-03-01", "Bank Lazur · savings account LN88 1020 0044 7310", """
Holders: Anna Lis and Tomasz Lis. Interest at 3.10 % a year on the daily balance, over 365 days, credited on the last day of each quarter less the 19 % tax. The bank may change the rate; the rate in force is shown on each statement.
""")
    doc("A-D32", "Bank Lazur SA", "2023-12-01", "2023-12-04", "Bank Lazur · rate notice", """
From 1 January 2024 the rate on your savings account LN88 1020 0044 7310 is 4.20 % a year.
""")
    doc("A-D33", "Bank Wrzos SA", "2025-01-02", "2025-01-02", "Bank Wrzos · standing orders on LN27 … 0412", """
| to | for | amount (DUK) | day | from | until |
|---|---|---|---|---|---|
| Jerzy Grab | rent | 3 350.00 | 1st | 2025-01-01 | — |
| Żłobek Pszczółka | nursery | 1 250.00 | 5th | 2024-09-05 | — |
| Fala Telekom | internet and phone | 139.99 | 20th | 2020-01-20 | — |
| Elektro Bór Raty | instalments | 380.00 | 25th | 2024-10-25 | 2025-09-25 |

A standing order due on a day that is not a business day is executed on the next business day.
""")
    doc("A-D34", "Elektro Bór", "2024-09-28", "2024-09-28", "Instalment purchase", """
A washing machine and a fridge, 4 560.00 DUK, in 12 interest-free instalments of 380.00 DUK on the 25th, October 2024 – September 2025.
""")
    w = sim["book"].withdrawal
    doc("A-D35", "Brzask TFI", "2024-03-07", "2024-03-08", "Brzask · withdrawal from the wrapped account", f"""
Request of 5 March 2024 to withdraw 20 000.00 DUK. Redeemed on 5 March 2024: units of BRZ-BND-A worth {sp(w[1])} DUK, paid on 7 March 2024 to LN27 … 0412. The amount withdrawn is within the contributions not yet withdrawn; no tax is withheld (PIT-9.4, .5).
""")
    doc("A-D36", "Kancelaria Notarialna Brzoza", "2025-06-12", "2025-06-13", "Notary · payout under the division of an estate", f"""
Under the agreement of 12 June 2025 dividing the estate of Zbigniew Sowa, Anna Lis receives {sp(a.ESTATE_PAYOUT[1])} DUK for her share, paid from escrow to her account LN61 … 5503 on 16 June 2025.
""")
    gt = a.gift_tax(known=ALL_KNOWN)  # R1: the generator writes the tax decision, seeing every figure
    doc("A-D38", "Revenue Office, Dębowo", "2024-06-05", "2024-06-10", "Revenue Office · decision on gift tax IGT/2024/0611", f"""
Anna Lis owes gift tax of {sp(gt['tax'])} DUK on the acquisition of 12 September 2023 from Halina Sowa, added to the acquisition of 30 April 2021 from the same person (IGT-3.1); the report of 14 March 2024 was filed after 12 March 2024 (IGT-4.2). Payable within 14 days of delivery.
""")
    doc("A-D37", "Bank Wrzos SA", "2025-06-02", "2025-06-02", "Bank Wrzos · savings account LN61 1140 2004 0000 3402 9921 5503", """
Holder: Anna Lis. Opened 2 June 2025. Interest at 2.40 % a year on the daily balance, over 365 days, credited on the last day of each month less the 19 % tax.
""")


def main():
    sim = a.simulate(known=ALL_KNOWN)  # R1: the generator writes the records of the world, seeing every figure and price
    book = sim["book"]
    joint_statements(sim)
    wrapped(book)
    rzeka(book, "RZK-T", "4471-02", "Tomasz Lis")
    rzeka(book, "RZK-A", "4471-07", "Anna Lis")
    kopia(book)
    wcp(sim, "T", "Tomasz Lis", "Nurt Software")
    wcp(sim, "A", "Anna Lis", "Kalina Logistyka")
    lazur_file(lazur())
    ania_savings()
    documents(sim)


if __name__ == "__main__":
    main()
