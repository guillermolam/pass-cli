# `session` command

Manage the session lock to prevent unauthorized access to your Proton Pass CLI session with a PIN code.

## Synopsis

```bash
pass-cli session lock [--lock-time SECONDS]
pass-cli session unlock
pass-cli session remove-lock
```

## Description

The `session` command lets you add a PIN-based lock to your active session. When the session is locked, all operations
that require the Proton Pass API are blocked until you unlock it with the correct PIN. This is useful when you want to
keep your session authenticated but prevent anyone with access to your terminal from running commands.

The lock is enforced server-side: even if local state is tampered with, the Proton Pass API will reject requests until
the session is unlocked. The lock also auto-expires after the configured timeout, at which point the session becomes
unusable again without a PIN.

## Subcommands

### lock

Lock the current session with a PIN.

```bash
pass-cli session lock [--lock-time SECONDS]
```

You will be prompted to enter a PIN. The PIN is not stored anywhere, it is sent to the Proton Pass API to establish the
lock. You must use the same PIN to unlock or remove the lock later.

**Options:**

- `--lock-time SECONDS` Time in seconds before the session auto-unlocks. Must be between 30 and 900. Default: `300` (5
  minutes).

**Examples:**

```bash
# Lock with the default 5-minute timeout
pass-cli session lock
# Enter PIN:
# Session locked successfully

# Lock with a custom 10-minute timeout
pass-cli session lock --lock-time 600
# Enter PIN:
# Session locked successfully
```

---

### unlock

Unlock a locked session using the PIN set at lock time.

```bash
pass-cli session unlock
```

You will be prompted for the PIN. On success, the session is restored to normal operation. This command fails if the
session is not currently locked.

**Examples:**

```bash
pass-cli session unlock
# Enter PIN:
# Session unlocked successfully
```

---

### remove-lock

Remove the session lock entirely, so no PIN is required going forward.

```bash
pass-cli session remove-lock
```

You will be prompted for the current PIN to confirm the removal. After this, the lock is deleted from the server and the
session operates normally without any PIN requirement.

**Examples:**

```bash
pass-cli session remove-lock
# Enter PIN:
# Session lock removed successfully
```

## Checking lock status

```bash
pass-cli info
```

The output includes a `Session has lock` field that shows whether the current session has an active lock. Having a
session lock does not mean that the session is locked at this moment. It means that if unused it will lock
automatically.

## Security considerations

- **PIN strength** Choose a PIN that is not trivially guessable. There is no minimum length enforced by the CLI, but a
  longer PIN is harder to brute-force.
- **Auto-unlock timeout** Keep `--lock-time` short on shared or unattended systems. The default of 300 seconds is a
  reasonable balance for interactive use.
- **Session vs. logout** Locking a session is not a substitute for `logout`. A locked session is still authenticated;
  it is just gated by the PIN. Use `logout` when you want to fully terminate the session.
- **PIN not stored** The PIN is never written to disk or the keyring. If you forget it, you cannot unlock or remove
  the lock until it auto-expires. You will need to log out and log in again.
