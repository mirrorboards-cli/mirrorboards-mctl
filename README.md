# mctl-next

Deklaratywny menedżer wielu repozytoriów. Workspace jest zawsze w jednym z dwóch stanów:
**BASE** (repa na branchach z profilu) albo **SESSION** (repa odbite na branche sesyjne).
Nie ma trzeciego stanu — i to jest cała filozofia tego narzędzia.

```
$ mctl-next --help

mctl — multi-repo workspaces, sessions and cross-repo PRs

  Your workspace is always in exactly one of two states:

    BASE      every repo on the branch your profile says (e.g. dev)
    SESSION   every repo forked onto <base>-<id> (e.g. dev-0x8ad4f2)

  Features go through sessions; quick fixes go straight in with save.
  Running mctl with no arguments opens the interactive TUI.

USAGE
  mctl [COMMAND] [OPTIONS]

SETUP (once per machine)
  init <git-url>         Clone the .mirror catalog repo; everything derives from it
  use [<group>...]       Pick which repo groups are active here (no args: picker)

BASE
  apply [--yes]          Reconcile disk with desired state: clone missing repos,
                         switch branches, prune deselected. Always shows the
                         plan first and asks; never touches dirty repos.
  status                 One screen: current state (base/session), branch drift,
                         dirty files, ahead/behind, open sessions

ANYWHERE
  save -m "<msg>"        Commit + push exactly where you stand, repos with
                         changes only. On BASE: the quick-fix path — straight
                         to the branch, no session, no PR. In a SESSION: a
                         checkpoint — pushes session branches without opening
                         or updating any PRs (that's what flush is for).

SESSIONS (the daily loop)
  start [--on <id>]      Open a session: fork ALL active repos onto <base>-<id>.
                         You don't declare what you'll touch — selection happens
                         at flush. --on stacks on another session and the branch
                         name nests: dev-0x123 -> dev-0x123-0x456. The name IS the
                         tree — plain `git branch` shows the whole ancestry.

                         Session ids always carry the 0x prefix (0x + hex, e.g.
                         0x8ad4f2). It marks a branch as machine-made — a human
                         would never name one like that — and since base branch
                         names contain dashes themselves, the first "-0x" is the
                         only unambiguous place where base ends and the session
                         lineage begins. Commands accept ids with or without 0x.
  flush -m "<msg>"       Publish the session: commit + push every repo that
                         changed, open (or update) one PR per repo and a meta-PR
                         in .mirror that links them all. You stay on the session.
  sessions [--tree]      List open sessions — yours and the team's, with PR and
                         review status. --tree shows the stack hierarchy.
  checkout <id> | base   Switch the whole workspace to a session (also someone
                         else's — pulls review commits) or back to base
  restack [<id>]         Pull parent changes down into a session, all repos
  ship <id> [--retarget] MERGE THE SESSION'S PRs (what gh pr merge would do,
                         repo by repo, in changeset order), then the meta-PR,
                         then delete session branches and move you onto the
                         freshly pulled parent. Requires a flushed session —
                         ship merges the PRs that flush opened; it refuses if
                         there is unflushed work or open child sessions.
                         Children keep their names after a parent ships — the
                         name records where a session was born from (lineage),
                         the changeset tracks where it merges to (target).
  discard [<id>]         Abandon a session; deletes its branches (warns first)

SHARING (you ↔ team)
  publish <profile>      Save current groups + base branches as a named profile
                         in .mirror (commit + push). Share intent, not checkouts.
  follow <profile>       Derive your BASE from a published profile. Their
                         updates arrive at your next apply; onboarding is
                         `follow` + `apply`.

RULES
  1. Work is sacred     — mctl never switches a dirty repo or deletes
                          unpushed commits. No exceptions, no silent force.
  2. No third state     — you are on BASE or in a SESSION; status always
                          knows which. Leaving base without a session is drift
                          and gets flagged.
  3. Nothing without a plan — every disk mutation is printed before it runs.

FILES
  mirror.toml            catalog: groups, orgs, path convention   (in .mirror)
  profiles/*.toml        published profiles                       (in .mirror)
  changesets/*.toml      one file per flushed session: PR list,
                         merge order, stack parent, rev lock      (in .mirror)
  .mirror/state.toml     this machine: active groups, followed
                         profile, current session                 (gitignored)
```

## Przykład: jeden dzień

```
$ mctl start                          # sesja 0x8ad4f2, wszystkie repa odbite
  ... vibe coding w 3 repach ...
$ mctl save -m "wip"                  # checkpoint: push na branche sesji, zero PR-ów
  ... dalej ...
$ mctl flush -m "Fix na xAuth"        # 2 repa się zmieniły → 2 PR-y + meta-PR
  ... review, poprawki ...
$ mctl flush -m "review fixes"        # te same PR-y, zaktualizowane
$ mctl ship 0x8ad4f2                    # merge, sprzątnięcie, jesteś na base
```

Szybki fix bez ceremonii:

```
$ mctl save -m "hotfix: typo w configu"   # prosto na dev, bez sesji i PR-ów
```

## Czego tu celowo NIE ma

Wycięte z pierwszej wersji — sesje albo `apply` załatwiają te przypadki, a jak
zabolí, dodamy:

- **`switch` / override'y / `adopt`** — jedyna droga poza base to sesja; ręczne
  żonglowanie branchami per repo to wracanie do chaosu, który nas tu przywiódł
- **`plan`** — wbudowany w `apply` (i w każdą mutację)
- **`begin` / `propose`** — `flush` robi oba naraz (`save` zostało: szybkie
  fixy na base i checkpointy w sesji)
- **`snapshot` / `lock`** — changeset każdej sesji i tak niesie lock rewizji
- **`worktree`** — dwie sesje obok siebie fizycznie; nice-to-have, nie fundament
- **`add` / `remove` / `show` / `validate` / `config`** — katalog edytuje się
  jak kod: w plikach `.mirror`, przez PR
