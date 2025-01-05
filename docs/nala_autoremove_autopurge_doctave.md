
# nala-autoremove/autopurge

## NAME

`nala-autoremove/autopurge` - Autoremove or autopurge packages that are no longer needed.

## SYNOPSIS

```bash
nala autoremove [--options]
nala autopurge [--options]
```

## DESCRIPTION

Automatically remove or purge packages that are no longer needed.

Packages that are no longer needed are typically dependencies that were automatically installed and are no longer required by any package.

The `autoremove` and `autopurge` documentation are combined because they are nearly identical.  
`nala autopurge` is essentially `nala autoremove --purge`.

## OPTIONS

### `--config`
Purge configuration files for packages that are no longer installed.

---

### `--purge`
Purge any packages that would be removed during the transaction.

---

### `--debug`
Print helpful information for troubleshooting.  
If submitting a bug report, rerun the command with `--debug` and include the output for developers.

---

### `--raw-dpkg`
Force `nala` not to format `dpkg` output.

#### Detailed Explanation:
- `nala` forks a TTY instead of a PTY for `dpkg`.
- Disables progress bars for `dpkg`.
- The output language is not forced to English.

---

### `-d`, `--download-only`
Only retrieve packages, without unpacking or installing them.

---

### `--remove-essential`
Allow the removal of essential packages.  
**Warning:** This is dangerous, but the option is available.

---

### `--assume-yes`, `--assume-no`

#### `-y`, `--assume-yes`
Automatically answer *yes* to prompts.  
If the configuration option `assume_yes` is enabled, this resets it to default behavior.

#### `-n`, `--assume-no`
Automatically answer *no* to prompts.

---

### `--simple`, `--no-simple`

#### `--simple`
Display a simpler and more condensed transaction summary.

#### `--no-simple`
Display the standard table transaction summary with more information.  
(Default)

---

### `-o`, `--option`
Set options to pass through to `apt`, `nala`, or `dpkg`.

Examples:
- Force `dpkg` to install new config files without prompting:  
  ```bash
  nala install --option Dpkg::Options::="--force-confnew"
  ```
- Disable scrolling text for `nala`:  
  ```bash
  nala install --option Nala::scrolling_text="false"
  ```
- Allow `nala` to update unauthenticated repositories:  
  ```bash
  nala install --option APT::Get::AllowUnauthenticated="true"
  ```

---

### `-v`, `--verbose`
Disable scrolling text and print additional information.

---

### `-h`, `--help`
Display this manual page.

---

### `--update`, `--no-update`

#### `--update`
Update the package list before performing the requested operation.  
Example:  
```bash
nala install --update neofetch
```
is equivalent to:  
```bash
apt update && apt install neofetch
```

#### `--no-update`
Do **NOT** update the package list before performing the requested operation.  
(Default for `install`, `remove`, `purge`, `autoremove`, `autopurge`)

---

### `--fix-broken`, `--no-fix-broken`

#### `--fix-broken`
Attempt to fix broken packages.  
(Default)

#### `--no-fix-broken`
Prevent `nala` from performing extra checks.  
**Note:** This may result in a broken installation.

To fix broken packages:  
```bash
nala install --fix-broken
```

## COPYRIGHT

Copyright (C) 2021, 2022 Blake Lee
