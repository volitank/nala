
---
title: nala-autoremove/autopurge
---

# NAME

**nala-autoremove/autopurge** - Autoremove or autopurge packages that are no longer needed

# SYNOPSIS

```bash
nala autoremove [--options]

nala autopurge [--options]
```

# DESCRIPTION

Automatically remove or purge any packages that are no longer needed.

Packages that are no longer needed are usually dependencies that were automatically installed and are no longer required by any other package.

The **autoremove** and **autopurge** documentation are combined as they are nearly identical.  
**nala autopurge** is essentially **nala autoremove** with the `--purge` option.

# OPTIONS

### **--config**
Purge configuration files for packages that are no longer installed.

---

### **--purge**
Purge any packages that would be removed during the transaction.

---

### **--debug**
Print helpful information for troubleshooting issues.  
If you're submitting a bug report, rerun the command with `--debug` and include the output for developers. It may help.

---

### **--raw-dpkg**
Force **nala** not to format **dpkg** output.  
This disables all formatting, making the output look as if **apt** were used.

#### Detailed Explanation:
- **nala** forks a TTY instead of a PTY for **dpkg**.
- No progress bar for **dpkg** will be displayed.
- The output language will not be forced to English.

---

### **-d**, **--download-only**
Packages are only retrieved, not unpacked or installed.

---

### **--remove-essential**
Allow the removal of essential packages.  
This is very dangerous, but the option is available.

---

### **--assume-yes**, **--assume-no**

#### *-y*, *--assume-yes*
Automatically select *yes* for prompts requiring input.  
If the configuration option *assume_yes* is true, this switch resets it to default behavior.

#### *-n*, *--assume-no*
Automatically select *no* for prompts requiring input.

---

### **--simple**, **--no-simple**

#### *--simple*
Show a more simple and condensed transaction summary.

#### *--no-simple*
Show the standard table transaction summary with more information (default).

---

### **-o**, **--option**
Set options to pass through to **apt**, **nala**, or **dpkg**.

Examples:
- Force **dpkg** to install new config files without prompting:  
  ```bash
  nala install --option Dpkg::Options::="--force-confnew"
  ```
- Disable scrolling text for **nala**:  
  ```bash
  nala install --option Nala::scrolling_text="false"
  ```
- Allow **nala** to **update** unauthenticated repositories:  
  ```bash
  nala install --option APT::Get::AllowUnauthenticated="true"
  ```

---

### **-v**, **--verbose**
Disable scrolling text and print extra information.

---

### **-h**, **--help**
Shows this man page.

---

### **--update**, **--no-update**

#### *--update*
Update the package list before the requested operation.  
Example:  
```bash
nala install --update neofetch
```
is equivalent to:  
```bash
apt update && apt install neofetch
```

#### *--no-update*
Do **NOT** update the package list before the requested operation.  
[Default for: **install**, **remove**, **purge**, **autoremove**, **autopurge**]

---

### **--fix-broken**, **--no-fix-broken**

#### *--fix-broken*
Attempt to fix broken packages (default).

#### *--no-fix-broken*
Prevent **nala** from performing extra checks.  
*This can result in a broken install!*

To fix broken packages:  
```bash
nala install --fix-broken
```

# COPYRIGHT

Copyright (C) 2021, 2022 Blake Lee
