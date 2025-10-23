# Login and authentication

This document explains how to authenticate with the Proton Pass CLI, including all available options for providing credentials and configuring the environment.

## Basic login

To log in with your Proton account:

```bash
pass-cli login <username>
```

Where `<username>` is your Proton account email address.

## Authentication flow

The login process follows these steps:

1. **Password authentication** - You'll be prompted for your Proton account password
2. **Two-factor authentication** (if enabled) - Support for TOTP codes and FIDO2/WebAuthn hardware keys
3. **Extra password** (if required) - Proton Pass users can configure their accounts to require an additional Pass-specific password
4. **Initial setup** - The CLI performs first-time setup and creates a default vault if none exists
5. **Permission check** - Verifies that your account is authorized to use the CLI (see `no-login-restriction` flag in `[00-BUILD.md](./00-BUILD.md) if you see this error)

## Providing credentials

For each authentication parameter, the CLI checks for values in this order:

1. **Environment variable** - Direct value
2. **File referenced by environment variable** - Path to file containing the value
3. **Interactive prompt** - If not found in env vars, prompts the user

### Password

**Interactive (default):**

```bash
pass-cli login user@proton.me
# You will be prompted: Enter password:
```

**Via environment variable:**

```bash
export PROTON_PASS_PASSWORD='your-password'
pass-cli login user@proton.me
```

**Via file:**

```bash
echo 'your-password' > /secure/password.txt
export PROTON_PASS_PASSWORD_FILE='/secure/password.txt'
pass-cli login user@proton.me
```

### Two-factor authentication (TOTP)

If your account has TOTP enabled:

**Interactive (default):**

```bash
pass-cli login user@proton.me
# After password, you'll be prompted: Enter TOTP:
```

**Via environment variable:**

```bash
export PROTON_PASS_TOTP='123456'
pass-cli login user@proton.me
```

**Via file:**

```bash
echo '123456' > /secure/totp.txt
export PROTON_PASS_TOTP_FILE='/secure/totp.txt'
pass-cli login user@proton.me
```

### FIDO2 / WebAuthn

If you have a hardware security key (YubiKey, etc.), the CLI will detect it and prompt you to interact with your device. This cannot be provided via environment variables as it requires physical interaction.

If both TOTP and FIDO2 are available, you'll be presented with a choice:

```
Multiple 2FA methods available:
1) TOTP
2) FIDO
Select authentication method:
```

### Extra password

Some Proton Pass accounts require an additional password (separate from your account password). This is the Pass-specific access password.

**Interactive (default):**

```bash
pass-cli login user@proton.me
# If required, you'll be prompted: Enter Pass extra password:
```

**Via environment variable:**

```bash
export PROTON_PASS_EXTRA_PASSWORD='your-extra-password'
pass-cli login user@proton.me
```

**Via file:**

```bash
echo 'your-extra-password' > /secure/extra-password.txt
export PROTON_PASS_EXTRA_PASSWORD_FILE='/secure/extra-password.txt'
pass-cli login user@proton.me
```

You have 3 attempts to enter the correct extra password before the CLI logs out.

## Environment configuration

### Selecting the API environment

By default, the CLI connects to the production Proton API. You would be able change this with the `ENVIRONMENT` variable:

**Production (default):**

```bash
# No variable needed, or explicitly:
export ENVIRONMENT=prod
pass-cli login user@proton.me
```

Other environment variable values are meant to be used by Proton internal developers to access internal test environments or custom backends.

When using `localhost`, the CLI disables TLS certificate verification automatically.

### Configuring logging

Control log output verbosity with these environment variables:

**Pass CLI logging:**

```bash
# Levels: trace, debug, info, warn, error, off
export PASS_LOG_LEVEL=debug
pass-cli login user@proton.me
```

**Muon (network library) logging:**

```bash
# Levels: trace, debug, info, warn, error, off
export MUON_LOG_LEVEL=info
pass-cli login user@proton.me
```

By default:
- In debug builds: `PASS_LOG_LEVEL=debug`
- In release builds: `PASS_LOG_LEVEL=off`
- `MUON_LOG_LEVEL=off` unless explicitly set

Logs include file names and line numbers for debugging.

### Session storage directory

By default, session data is stored in:
- **macOS**: `~/Library/Application Support/proton-pass-cli/.session/`
- **Linux**: `~/.local/share/proton-pass-cli/.session/`

If desired, you can override this with:

```bash
export PROTON_PASS_SESSION_DIR='/custom/path'
pass-cli login user@proton.me
```

### Custom app header

The CLI identifies itself to the API with an app header. It's not meant to be overriden, but for internal Proton developers, we may override it with:
```bash
export PROTON_PASS_APP_HEADER='custom-client@1.0.0'
pass-cli login user@proton.me
```

Default: `cli-pass@<version>`

## Checking authentication status

After logging in, verify your session:

```bash
pass-cli info
```

This shows:
- Your username
- Account details
- Active session information

## Logout

To log out and clear your session:

```bash
pass-cli logout
```

To force logout even if remote logout fails:

```bash
pass-cli logout --force
```

## Example: Fully automated login

For scripting and automation, you can provide all credentials via environment variables:

```bash
#!/bin/bash

export PROTON_PASS_PASSWORD='your-password'
export PROTON_PASS_TOTP='123456'
export PROTON_PASS_EXTRA_PASSWORD='your-extra-password'
export PASS_LOG_LEVEL=error  # Reduce noise

pass-cli login user@proton.me
```

Or using files:

```bash
#!/bin/bash

export PROTON_PASS_PASSWORD_FILE='/secure/creds/password.txt'
export PROTON_PASS_TOTP_FILE='/secure/creds/totp.txt'
export PROTON_PASS_EXTRA_PASSWORD_FILE='/secure/creds/extra-password.txt'

pass-cli login user@proton.me
```

## Security notes

- All credentials are transmitted over TLS (unless using localhost with disabled verification)
- Password and extra password prompts use secure input (characters are not echoed)
- Session tokens are stored encrypted on disk (see `02-PERSISTENCE.md` for details)

