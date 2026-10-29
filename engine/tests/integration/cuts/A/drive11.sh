#!/bin/sh
D=engine/tests/integration/cuts/A
R=$D/run11.sh
$R $D/base.txt q01-04 a01_2021 a01_2022 a01_2022_causes a01_2023 a01_without_2022 a01_without_2023 a02 a03 a03_advances a03_without a04_months a04_total & $R $D/base.txt q06-09 a06 a07 a08 a08_steps a09 a09_steps & wait
$R $D/base.txt q10-11-05a a10 a11 a11_known_part a05_tomek a05_tomek_2024 a05_wrapped & $R $D/base.txt q12 a12_base a12_t25_base a12_crash a12_t25_crash a12_flat a12_t25_flat a12_switch a12_t25_switch & wait
$R $D/diag05.txt d05 mca05_bal mca05_plain & $R $D/diag10.txt d10 mca_a10_bal mca_a10_explain & wait
$R $D/cut05.txt c05-ania a05_ania & $R $D/cut05.txt c05-why a05_ania_why & wait
$R $D/cut05d.txt c05-down mca05_down & $R $D/cut05d.txt c05-up mca05_up & wait
for f in f1 f2 f3 f4 MC-A-F1 MC-A-F1-pinned; do engine/tools/parsecheck/.venv/bin/python -I -B tmp/design2/scratch/lead/snap11/refcore run $D/min/$f.tally > $D/out11/min-$f.out 2>&1; done
(timeout 330 $R $D/base.txt q05-ania a05_ania; echo "exit=$?" > $D/out11/q05-ania.done) & (timeout 330 $R $D/base.txt q05-why a05_ania_why; echo "exit=$?" > $D/out11/q05-why.done) & wait
echo all > $D/out11/ALL.done
