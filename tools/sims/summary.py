#!/usr/bin/env python3
# summary.py DIR...: pillar means and per-house economy for each directory of watch runs (house-SEED.json).
import json, sys, glob, statistics as st

def load(path):
    return json.loads(open(path).read().split('\n')[0])

def pillars(d):
    last = d['series'][-1]
    act = last.get('activity', {})
    kinds = dict(k.rsplit(':', 1) for k in d.get('kinds', []))
    kinds = {k: int(v) for k, v in kinds.items()}
    bf = d.get('byFaction', [])
    st = d.get('structures', {}).get('byFaction', [])
    fielded = {}
    for f in bf:
        for t, n in (f.get('fielded') or {}).items():
            fielded[t] = fielded.get(t, 0) + n
    mixed = sum(1 for f in bf if len([t for t, n in (f.get('fielded') or {}).items() if t not in ('miner', 'hauler') and n > 0]) >= 3)
    e = last.get('energy') or {}
    out = {
        'battles': {'kills': act.get('kills'), 'battles': act.get('battles'), 'captures': act.get('captures'), 'TAKE': kinds.get('TAKE', 0),
                    'mixedArmies': mixed, 'fallen': len(d.get('fallen', [])), 'capitalSieges': len(d.get('capitalSieges', [])),
                    'fielded': fielded},
        'sea': {**d.get('sea', {})},
        'trade': {'deals': act.get('deals'), 'contracts': act.get('contracts'), 'TRADE': kinds.get('TRADE', 0), 'DELIVERY': kinds.get('DELIVERY', 0),
                  'redeemed': round(d.get('logistics', {}).get('redeemed', 0) / 1e6, 1)},
        'air': {'AIR': kinds.get('AIR', 0), 'airUnitsEnd': sum(f.get('air') or 0 for f in bf),
                'flight': sum(1 for f in bf if 'flight' in (f.get('techs') or [])), 'airWeapons': sum(1 for f in bf if 'airWeapons' in (f.get('techs') or []))},
        'diplomacy': {'PACT': kinds.get('PACT', 0), 'MSG': kinds.get('MSG', 0), 'BETRAY': kinds.get('BETRAY', 0), 'pacts': d.get('pacts'), 'betrays': d.get('betrays')},
        'building': {'constructions': sum(s['constructions'] for s in st), 'completions': sum(s['completions'] for s in st),
                     'destructions': sum(s['destructions'] for s in st), 'standing': sum(s['standing'] for s in st)},
        'energy': {k: e.get(k) for k in ('blackouts', 'cuts', 'captured', 'made', 'used', 'meltdowns', 'ignited', 'deals')} if e else None,
        'shares': [round(f.get('share', 0), 3) for f in bf],
        'winner': d.get('winner'), 'alive': d.get('alive'), 'techs': d.get('techs'),
    }
    if e:
        out['energy']['byFaction'] = [{'f': x['faction'], 'powered': x['powered'], 'supply': x['supply'], 'demand': x['demand'], 'rated': x['rated'],
                                       'storage': x['storage'], 'sold': x['sold'], 'bought': x['bought']} for x in e.get('factions', [])]
        pts = [pt.get('energy') for pt in d['series'] if pt.get('energy')]
        avg = [sum(f['powered'] for f in q['factions']) / max(1, len(q['factions'])) for q in pts]
        out['energy']['poweredMean'] = round(sum(avg) / max(1, len(avg)), 3)
        out['energy']['poweredByMin'] = [round(a, 2) for a in avg[::max(1, len(avg) // 12)]]
        tot = {}
        for x in e.get('factions', []):
            for k, v in (x.get('rated') or {}).items(): tot[k] = round(tot.get(k, 0) + v, 1)
        out['energy']['ratedTotal'] = tot
    return out

def ranks(v):
    o = sorted(range(len(v)), key=lambda i: v[i]); r = [0.0] * len(v); i = 0
    while i < len(o):
        j = i
        while j + 1 < len(o) and v[o[j + 1]] == v[o[i]]: j += 1
        for k in range(i, j + 1): r[o[k]] = (i + j) / 2
        i = j + 1
    return r
def spearman(a, b):
    ra, rb = ranks(a), ranks(b); n = len(a); ma, mb = sum(ra) / n, sum(rb) / n
    num = sum((x - ma) * (y - mb) for x, y in zip(ra, rb)); den = (sum((x - ma) ** 2 for x in ra) * sum((y - mb) ** 2 for y in rb)) ** .5
    return num / den if den else 0.0

def run(f, house=3.0):
    d = load(f); p = pillars(d); b = d.get('battles') or {}
    pts = [s for s in d['series'] if s.get('t', 0) >= 600 and s.get('f')]
    inc = st.mean(sum(x[0] for x in s['f'] if x) for s in pts)
    cores = st.mean(sum(x[2] for x in s['f'] if x) for s in pts)
    total = sum(sum(x[0] for x in s['f'] if x) for s in d['series'] if s.get('f')) * 60
    bf = d['structures'].get('byFaction') or []
    if isinstance(bf, dict): bf = list(bf.values())
    build = sum(x.get('constructionCost', 0) + x.get('operationalCost', 0) for x in bf) / total if total else 0
    r = {'fights': p['battles']['battles'], 'kills': p['battles']['kills'], 'captures': p['battles']['captures'],
         'pitched': b.get('pitchedPerHour', b.get('pitched')), 'roles': b.get('rolesMean'), 'mixedBoth': b.get('mixedBoth'),
         'swings': b.get('swingsMean'), 'battleSecs': b.get('secsMedian'),
         'sunk': p['sea'].get('sunk', 0), 'broadsides': p['sea'].get('broadsides', 0), 'prizes': p['sea'].get('prizes', 0),
         'deals': p['trade']['deals'], 'deliveries': p['trade']['DELIVERY'],
         'air': p['air']['AIR'], 'flight': p['air']['flight'],
         'aircraft': sum((x.get('fielded') or {}).get(k, 0) for x in d['byFaction'] for k in ('wing', 'kite', 'interceptor', 'aeronaut')),
         'pacts': p['diplomacy']['PACT'], 'msgs': p['diplomacy']['MSG'], 'betrayals': p['diplomacy']['BETRAY'],
         'built': p['building']['constructions'], 'standing': p['building']['standing'],
         'income': inc, 'cores': cores, 'perCore': inc / cores if cores else 0, 'buildShare': build}
    e = p.get('energy') or {}
    if e.get('byFaction'):
        made = {}; met = drawn = fromplants = 0.0
        for s in d['series']:
            for x in (s.get('energy') or {}).get('factions', []):
                fi = x['faction']; dm = x['demand']; m = x['powered'] * dm
                drawn += dm; met += m; fromplants += max(0.0, m - min(m, house))
                made.setdefault(fi, []).append(max(0.0, x['supply'] - house))
        last = d['series'][-1]['f']; alive = [i for i, v in enumerate(last) if v]
        pw = [sum(made.get(i, [0])) / max(1, len(made.get(i, [0]))) for i in alive]
        cores_end = [last[i][2] for i in alive]
        best = alive[max(range(len(alive)), key=lambda k: pw[k])]
        r.update({'powered': e.get('poweredMean'), 'plants': fromplants / max(met, 1e-9), 'dark': 1 - met / max(drawn, 1e-9),
                  'rho': spearman(pw, cores_end), 'top3': 1.0 if best in sorted(alive, key=lambda i: -last[i][2])[:3] else 0.0,
                  'blackouts': e.get('blackouts'), 'meltdowns': e.get('meltdowns'), 'ignited': 1.0 if e.get('ignited') else 0.0,
                  'rated': e.get('ratedTotal') or {}})
    r['powerSold'] = sum(x.get('sold') or 0 for x in (e.get('byFaction') or []))
    r['crowned'] = 1.0 if d.get('winner') is not None else 0.0
    shares = [x.get('share', 0) for x in d['byFaction'][:8]]
    r['leader'] = max(range(len(shares)), key=lambda i: shares[i]) if shares else -1
    r['contracts'] = (d['series'][-1].get('activity') or {}).get('contracts', 0)
    return r

NAMES = ['Kublai', 'Arjuna', 'Templars', 'Atlantis', 'Fleet', 'Manifesto', 'NullPtr', 'Compute']
KEYS = ['runs', 'fights', 'kills', 'pitched', 'roles', 'mixedBoth', 'swings', 'sunk', 'broadsides', 'prizes', 'deals', 'deliveries', 'air', 'aircraft',
        'pacts', 'msgs', 'built', 'income', 'buildShare', 'powered', 'plants', 'dark', 'rho', 'top3', 'blackouts', 'powerSold', 'contracts', 'crowned']

def houses(dd):
    agg = [dict(inc=0, n=0, stall=0, sn=0, op=0) for _ in range(8)]
    for f in sorted(glob.glob(dd + '/house-*.json')):
        try: d = json.loads(open(f).readline())
        except json.JSONDecodeError: continue
        s = d['series']
        for a, b in zip(s, s[1:]):
            if b['t'] < 600: continue
            sa = {x['faction']: x for x in a['structures']['byFaction']}
            sb = {x['faction']: x for x in b['structures']['byFaction']}
            for i, r in enumerate(b['f'][:8]):
                if not r: continue
                g = agg[i]; g['inc'] += r[0]; g['n'] += 1
                if r[1] >= 2: g['sn'] += 1; g['stall'] += r[0] < 15000
                if i in sa and i in sb: g['op'] += (sb[i]['operationalCost'] - sa[i]['operationalCost']) / (b['t'] - a['t'])
    return ' '.join('%s %dk/%d%%/%.1f%%' % (NAMES[i][:4], g['inc'] / max(g['n'], 1) / 1000, 100 * g['stall'] / max(g['sn'], 1), 100 * g['op'] / max(g['inc'], 1)) for i, g in enumerate(agg))

print('dir'.ljust(10), ' '.join(k[:7].rjust(7) for k in KEYS))
for dd in sys.argv[1:]:
    rows = []
    for f in sorted(glob.glob(dd + '/house-*.json')):
        try: rows.append(run(f))
        except (json.JSONDecodeError, KeyError, IndexError): print('skipped', f, file=sys.stderr)
    if not rows: continue
    m = {'runs': len(rows)}
    for k in rows[0]:
        if k == 'leader': continue
        vals = [r[k] for r in rows if isinstance(r.get(k), (int, float))]
        m[k] = st.mean(vals) if vals else None
    print(dd[-10:].ljust(10), ' '.join((f'{m[k]:7.2f}' if isinstance(m.get(k), float) and abs(m[k]) < 1000 else str(int(m[k]) if isinstance(m.get(k), float) else m.get(k))).rjust(7) for k in KEYS))
    print('  houses (income/stalled/structure upkeep share):', houses(dd))
    lead = {}
    for r in rows: lead[NAMES[r['leader']] if 0 <= r['leader'] < 8 else '?'] = lead.get(NAMES[r['leader']] if 0 <= r['leader'] < 8 else '?', 0) + 1
    print('  leaders at the end:', ' '.join('%s %d' % kv for kv in sorted(lead.items(), key=lambda kv: -kv[1])))
