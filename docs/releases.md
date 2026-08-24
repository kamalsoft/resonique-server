# Releases and Installation

Release artifacts are published to the repository’s GitHub Releases page.

Supported targets:

| Platform | Artifact |
|---|---|
| Windows x64 | `.zip` and PowerShell installer |
| macOS Apple Silicon | `.tar.xz` and shell installer |
| macOS Intel | `.tar.xz` and shell installer |
| Linux x64 glibc | `.tar.xz` and shell installer |
| Linux ARM64 glibc | `.tar.xz` and shell installer |
| Linux x64 musl | `.tar.xz` and shell installer |
| macOS/Linux | Homebrew formula |

## Install with the shell installer

```bash
curl --proto '=https' --tlsv1.2 -LsSf \
  https://github.com/kamalsoft/resonique-server/releases/latest/download/resonique-server-installer.sh \
  | sh
```

## Install with PowerShell

```powershell
irm https://github.com/kamalsoft/resonique-server/releases/latest/download/resonique-server-installer.ps1 | iex
```

For production environments, download the archive and verify
`SHA256SUMS` before installation.

## Verify an archive

```bash
shasum -a 256 -c SHA256SUMS
```

Release artifacts are built only after formatting, compilation, tests, Clippy,
and dependency-audit checks pass.