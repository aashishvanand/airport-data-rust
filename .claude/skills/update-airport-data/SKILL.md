---
name: update-airport-data
description: Sync data/airports.json from airport-data-js into airport-data-rust, regenerate derived data files, check dependencies, bump the patch version and release. Use when the user says the JS library has new airport data, asks to update/sync airports, or asks for a data release.
---

# Update airport data (airport-data-rust)

`airport-data-js` is the source of truth for the dataset. This repo ships a copy of it.
Work from the repo root (`/Users/aashishvanand/Code/airport-data-rust`).

## 1. Get the new data

```bash
git checkout main && git pull --ff-only
git -C ../airport-data-js fetch origin
git -C ../airport-data-js show origin/main:data/airports.json > data/airports.json
git diff --stat data/airports.json   # nothing changed? stop, there is nothing to release
```

## 2. Scan field types before touching code

Upstream sometimes changes value types (for example, `""` became `null` for missing
`elevation_ft` / `runway_length` in JS 4.0.0). Compare old against new for every field:

```bash
git show HEAD:data/airports.json > /tmp/old_airports.json
python3 - <<'EOF'
import json
from collections import Counter
o = json.load(open('/tmp/old_airports.json')); n = json.load(open('data/airports.json'))
print('count', len(o), '->', len(n))
for f in n[0]:
    co = Counter(type(a.get(f)).__name__ for a in o); cn = Counter(type(a.get(f)).__name__ for a in n)
    if co.keys() != cn.keys():
        print(f, dict(co), '->', dict(cn))
old = {(a['iata'], a['icao']) for a in o}
print('added', [(a['iata'], a['icao'], a['airport']) for a in n if (a['iata'], a['icao']) not in old])
EOF
```

If a field gains a new type (`NoneType`, `str` in a numeric field, and so on), check that the decoder below handles it
and add a test for it. Also read the top of `../airport-data-js/CHANGELOG.md` for data fixes and breaking type changes.

Decoder: the `deserialize_opt_i64` / `deserialize_opt_f64` / `deserialize_string_or_int` helpers in `src/lib.rs`.
They accept number, `""`, and `null` (through `Option::<NumOrStr>`). A type they don't cover makes the whole dataset
fail to load, so every test panics inside `once_cell`.

## 3. Regenerate derived files

Nothing to do here. `build.rs` gzips `data/airports.json` into `OUT_DIR` at build time.

## 4. Check for dependency updates

```bash
cargo update --dry-run    # review the changes
cargo update
```
Also check `.github/workflows/*.yml` action versions and any open dependabot PRs (`gh pr list`).

## 5. Bump the version (patch for data-only updates)

Edit `version` in `Cargo.toml`, then run `cargo check` so `Cargo.lock` picks it up.

## 6. Verify locally (same checks as CI)

```bash
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test
```

If a test fails, confirm the failure comes from a real data change (new airport count, corrected
coordinates, and so on) before you update the expected value.

## 7. Commit and push main, then wait for CI

Stage explicit paths only. Never use `git add .`, because there are untracked local files.

```bash
git add Cargo.toml Cargo.lock data/airports.json src/lib.rs
git commit -m "feat: sync airport data with airport-data-js and bump version to X.Y.Z"
git push origin main
gh run list --branch main --limit 1        # then: gh run watch <id> --exit-status
```

## 8. Release (only after CI on main is green; publishing is irreversible)

Merging into `release` triggers CI and publishes to crates.io:
```bash
git checkout release && git pull --ff-only && git merge --ff-only main && git push origin release
git checkout main
```
Check the result at https://crates.io/crates/airport-data.
