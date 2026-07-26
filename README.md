```bash
cargo install --git ssh://git@github.com/mirrorboards/mirrorboards-mctl.git --config net.git-fetch-with-cli=true
```

# mctl - Mirror Configuration Management Tool (Next)

Nowa wersja narzędzia CLI do zarządzania wieloma repozytoriami git zdefiniowanymi w pliku `mirror.toml`. Używa zewnętrznego `git` CLI zamiast libgit2, co rozwiązuje problemy z SSH agent na różnych systemach.

## Funkcjonalności

- **Workspaces** - grupowanie repozytoriów w logiczne przestrzenie
- **Wersjonowanie** - branch/rev/tag dla każdego repo
- **Includes** - komponowanie konfiguracji z wielu plików
- **Remote Config** - synchronizacja mirror.toml z remote repo
- **Snapshots** - tworzenie migawek z konkretnymi commit hashami

## Instalacja

```bash
cargo install --git ssh://git@github.com/mirrorboards/mirrorboards-mctl.git --config net.git-fetch-with-cli=true
```

## Szybki start

```bash
# Inicjalizacja
mctl init

# Dodawanie repozytoriów
mctl add git@github.com:org/repo.git --workspace api
mctl add git@github.com:org/lib.git --workspace api --workspace core

# Synchronizacja
mctl sync                    # wszystkie
mctl sync api                # tylko workspace api
mctl sync --create-missing-branches  # jeśli skonfigurowany branch nie istnieje zdalnie, utwórz go z domyślnego

# Status
mctl status
mctl status api --detailed

# Zapisywanie zmian
mctl save --message "Update"
mctl save api --message "Update API"

# Snapshot
mctl snapshot                # → mirror.snapshot.toml
mctl snapshot api            # tylko workspace api
```

## Format konfiguracji

```toml
# mirror.toml

# Opcjonalnie: include innych plików
[includes]
paths = [
    "teams/frontend.toml",
    "teams/backend.toml",
]

# Opcjonalnie: synchronizacja z remote
[remote]
git = "git@github.com:org/mirror-config.git"
branch = "main"

# Repozytoria
[[repositories]]
git = "git@github.com:org/api.git"
path = "services/api"
branch = "main"              # lub: rev = "abc123..." lub: tag = "v1.0.0"
workspaces = ["api", "core"]

[[repositories]]
git = "git@github.com:external/lib.git"
path = "external/lib"
tag = "v2.0.0"
skip-push = true             # read-only
workspaces = ["external"]
```

## Komendy

| Komenda | Opis |
|---------|------|
| `mctl init` | Inicjalizacja nowej konfiguracji |
| `mctl add <url>` | Dodanie repozytorium |
| `mctl list [workspace]` | Lista repozytoriów |
| `mctl remove <path>` | Usunięcie repozytorium |
| `mctl show <path>` | Szczegóły repozytorium |
| `mctl validate` | Walidacja konfiguracji |
| `mctl sync [workspace]` | Synchronizacja (clone/pull); `--create-missing-branches` tworzy branch z domyślnego, gdy skonfigurowany branch nie istnieje zdalnie |
| `mctl status [workspace]` | Status repozytoriów |
| `mctl diff [workspace]` | Diff zmian |
| `mctl save [workspace]` | Commit i push zmian |
| `mctl snapshot [workspace]` | Utworzenie snapshot |
| `mctl from-org <org>` | Wygenerowanie mirror.toml z organizacji GitHub (alias: `get-repos`) |
| `mctl config init <url>` | Inicjalizacja remote config |
| `mctl config pull` | Pobranie config z remote |
| `mctl config push` | Wysłanie config do remote |
| `mctl config diff` | Diff z remote config |

## Opcje globalne

- `--config <file>` - użyj innego pliku konfiguracji (default: `mirror.toml`)
- `--verbose` / `-v` - szczegółowy output
- `--no-color` - wyłącz kolory

## Workspaces

Workspaces pozwalają grupować repozytoria i wykonywać operacje tylko na wybranej grupie:

```bash
# Dodaj repo do wielu workspace'ów
mctl add git@github.com:org/shared.git --workspace api --workspace web

# Operacje na workspace
mctl sync api           # sync tylko api
mctl status web         # status tylko web
mctl save core          # save tylko core
```

## Includes

Komponuj konfigurację z wielu plików:

```toml
# mirror.toml
[includes]
paths = [
    "teams/frontend.toml",
    "teams/backend.toml",
]
```

## Remote Config

Synchronizuj konfigurację między maszynami:

```bash
# Ustaw remote
mctl config init git@github.com:org/mirror-config.git

# Push lokalnej konfiguracji
mctl config push -m "Update config"

# Pull na innej maszynie
mctl config pull
```

## Snapshot

Utwórz migawkę z dokładnymi commit hashami:

```bash
mctl snapshot                          # → mirror.snapshot.toml
mctl snapshot --output prod.toml       # → prod.toml
mctl snapshot api                      # tylko workspace api

# Przywróć ze snapshot
mctl --config mirror.snapshot.toml sync
```

## Generowanie z organizacji GitHub

Zbuduj `mirror.toml` na podstawie wszystkich repozytoriów organizacji (lub użytkownika). Komenda korzysta z GitHub CLI (`gh`), więc wykorzystuje jego uwierzytelnianie i paginację — musi być zainstalowane i zalogowane (`gh auth login`).

```bash
# Wypisz mirror.toml na stdout (pipeable)
mctl from-org holonym-foundation > mirror.toml

# Alias
mctl get-repos holonym-foundation

# Zapis bezpośrednio do pliku
mctl from-org holonym-foundation --output mirror.toml

# Przypisz wszystkie repo do workspace, użyj HTTPS, przypnij domyślny branch
mctl from-org holonym-foundation --workspace holonym --https --pin-branch
```

Domyślnie pomijane są repozytoria zarchiwizowane i forki (`--include-archived`, `--include-forks`, aby je dołączyć). Adresy SSH są używane domyślnie (`--https` dla HTTPS). Diagnostyka trafia na stderr, więc `> mirror.toml` daje czysty plik konfiguracyjny.

## License

MIT OR Apache-2.0
