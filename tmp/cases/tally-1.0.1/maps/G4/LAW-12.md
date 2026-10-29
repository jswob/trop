# Map · LAW-12 · probes/law/LAW-12.tally

step 1 · Kuba: exempt = close true and not (unreported and not minor) → tax 0.00 · lines 25–38, 50–54 · L §5.10 · same (close fires on grandchild; must_report is false for a minor, so late doesn't fire)
step 2 · Leon: exempt = close false … → tax 5400.00 · lines 25–38, 44–45, 50–54 · L §5.10 · same (close doesn't fire on nephew; late doesn't fire; base gives false; 0.12 × 45000)
ask returns: value · each_heir gives Kuba exempt true, 0.00; Leon exempt false, 5400.00
checks: parsecheck ok · loadcheck ok
note: the duty to report is its own def (must_report) so the minors' exception sits on the duty, not on exempt; a minors rule excepting late on exempt would be transitive over close (L §5.10 step 4) and would make Leon exempt
