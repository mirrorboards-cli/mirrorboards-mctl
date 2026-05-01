# Quick Task: komenda mctl status nie dziala zbyt dobrze po ostatnich zmianach. Przestalem widziec liste zmienionych plikow. przywroc poprzednie zachowanie

**Date:** 2026-05-01
**Branch:** main

## What Changed
- Restored the default `mctl status` table's changed-file list by replacing the compact `Changes` counts column with a multiline `Files` column.
- Preserved the newer sync-state column while showing up to 10 changed files with status prefixes and a truncation marker for longer lists.
- Added unit coverage for file-list rendering and truncation behavior.

## Files Modified
- `src/cli/commands/status.rs`
- `.gsd/quick/1-komenda-mctl-status-nie-dziala-zbyt-dobr/1-SUMMARY.md`

## Verification
- `cargo test`
- Manual smoke test with a temporary mirror config and dirty local repository confirmed `mctl status` output includes both modified and untracked file names.
