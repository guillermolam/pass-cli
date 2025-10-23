# Data operations and workflows

This document provides a practical guide to using the Proton Pass CLI for managing vaults, items, and secrets in your applications.

## Overview

The CLI operates on a hierarchical data model:
```
Account
└── Share (representing either Vaults or Items shared with you)
    └── Items (logins, notes, passwords, etc.)
        └── Fields (username, password, custom fields, etc.)
```

## Working with vaults

Vaults are containers for items. Each vault has a unique ID and is accessed through a unique share ID.

You can think of a Share as the relationship between a User and a Resource.
Many users can access the same vault (with the same VaultID), but each of them will do it based on their own Share.
Resources can be either Vaults or Items (a user can share with another user access to an entire Vault, or just to a specific Item). 

### List all vaults

```bash
pass-cli vault list
```

Output shows:
- Vault name
- Share ID (needed for other operations)
- Item count
- Owner information

### Create a vault

```bash
pass-cli vault create "Work Passwords"
```

This creates a new vault and returns its share ID:
```
Created vault with id: AbCdEf123456
```

**Note:** A default vault named "Personal" is automatically created on first login if no vaults exist.

### Get share list

```bash
pass-cli share list
```

This shows detailed information about all shares you have access to, both for Vaults and also Items shared with you.

## Working with items

Items are the actual credentials and data stored in vaults.

### List items in a vault

For listing items in a vault, you can use the Share if of the vault.

```bash
pass-cli item list --share-id AbCdEf123456
```

Or you can specify the vault name

```bash
pass-cli item list "Personal"
```

### Create a login item

**Basic creation:**
```bash
pass-cli item create login \
  --share-id AbCdEf123456 \
  --title "GitHub Account" \
  --username "octocat" \
  --password "secret123" \
  --url "https://github.com"
```

**Generate a random password:**
```bash
pass-cli item create login \
  --share-id AbCdEf123456 \
  --title "New Account" \
  --username "user@example.com" \
  --generate-password
```

**Generate password with custom settings:**
```bash
pass-cli item create login \
  --share-id AbCdEf123456 \
  --title "New Account" \
  --username "user@example.com" \
  --generate-password=32,uppercase,symbols
```

Format: `length,uppercase,symbols` (numbers are always included)

**Generate a passphrase:**

```bash
pass-cli item create login \
  --share-id AbCdEf123456 \
  --title "New Account" \
  --username "user@example.com" \
  --generate-passphrase=5
```

This generates a passphrase with 5 words.

### Create from template

Get the JSON template:
```bash
pass-cli item create login --get-template
```

Output:
```json
{
  "title": "",
  "username": null,
  "email": null,
  "password": null,
  "urls": []
}
```

Create from template file:
```bash
cat > login.json << EOF
{
  "title": "Production Database",
  "username": "dbadmin",
  "password": "verysecure",
  "urls": ["https://db.example.com"]
}
EOF

pass-cli item create login \
  --share-id AbCdEf123456 \
  --from-template login.json
```

Or from stdin:
```bash
echo '{"title":"Test","username":"user","password":"pass","urls":[]}' | \
  pass-cli item create login \
    --share-id AbCdEf123456 \
    --from-template -
```

### View an item

The CLI can print the full details of an item by specifying both the Share id and the Item id

```bash
pass-cli item view --share-id AbCdEf123456 --item-id XyZ789
```

And also by specifying the path in a URI format:

```bash
pass-cli item view "pass://Personal/TestItem"
```

### View an item field

The CLI can print a single field of an item by specifying the Share id, the Item id and the field name

```bash
pass-cli item view --share-id AbCdEf123456 --item-id XyZ789 --field password
```

And also by specifying the path in a URI format:

```bash
pass-cli item view "pass://Personal/TestItem/password"
```

### Delete an item

```bash
pass-cli item delete --share-id AbCdEf123456 --item-id XyZ789
```

## Secret references

The CLI uses a URL-like syntax to reference secrets stored in Pass:

```
pass://<vault-name-or-id>/<item-name-or-id>/<field-name>
```

Examples:
```
pass://Work/GitHub/password
pass://Personal/Email Login/username
pass://AbCdEf123456/XyZ789/password
pass://My Vault/My Item/My Custom Field
```

**Notes:**
- Vault and item can be referenced by name or ID
- Names with spaces are supported
- Field name must match exactly (case-sensitive)
- Common fields: `username`, `password`, `email`, `url`, `note`

## The `run` command

The `run` command executes a program with secrets injected as environment variables. It searches environment variables for secret references and resolves them before running the command.

### Basic usage

```bash
# Set environment variables with secret references
export DB_PASSWORD='pass://Production/Database/password'
export API_KEY='pass://Work/External API/api_key'

# Run a command
pass-cli run -- ./my-app
```

The application sees:

```bash
DB_PASSWORD='actual-secret-value'
API_KEY='actual-api-key-value'
```

### Using .env files

Create a `.env` file:

```bash
cat > .env << EOF
DB_HOST=localhost
DB_PORT=5432
DB_USERNAME=admin
DB_PASSWORD=pass://Production/Database/password
API_KEY=pass://Work/External API/api_key
API_SECRET=pass://Work/External API/secret
EOF
```

Run with the env file:
```bash
pass-cli run --env-file .env -- ./my-app
```

Multiple env files are supported:

```bash
pass-cli run \
  --env-file base.env \
  --env-file secrets.env \
  --env-file local.env \
  -- ./my-app
```

### Secret masking

By default, the `run` command masks secrets in stdout/stderr:

```bash
pass-cli run -- ./my-app
```

If the application logs `API_KEY: sk_live_abc123`, the output shows:

```
API_KEY: <concealed by Proton Pass>
```

Disable masking:

```bash
pass-cli run --no-masking -- ./my-app
```

### Running with arguments

Pass arguments to your application:
```bash
pass-cli run -- ./my-app --config production --verbose
```

### Interactive programs

The `run` command supports stdin/stdout/stderr forwarding, so interactive programs work normally:

```bash
pass-cli run -- python
```

### Signal handling

`Ctrl+C` (SIGTERM) is properly forwarded to the child process. The CLI waits for graceful shutdown before sending SIGKILL if needed.

## The `inject` command

The `inject` command processes template files and replaces secret references with actual values. It uses handlebars-style syntax.

### Template syntax

Use double braces to mark secret references:
```
{{ pass://vault/item/field }}
```

Create a template file:
```yaml
# config.yaml.template
database:
  host: localhost
  port: 5432
  username: {{ pass://Production/Database/username }}
  password: {{ pass://Production/Database/password }}

api:
  key: {{ pass://Work/API Keys/api_key }}
  secret: {{ pass://Work/API Keys/secret }}

# This comment with pass://fake/uri is ignored
# Only {{ }} wrapped references are processed
```

### Inject to stdout

```bash
pass-cli inject --in-file config.yaml.template
```

This prints the processed template to stdout.

### Inject to file

```bash
pass-cli inject \
  --in-file config.yaml.template \
  --out-file config.yaml
```

If the output file exists, add `--force`:

```bash
pass-cli inject \
  --in-file config.yaml.template \
  --out-file config.yaml \
  --force
```

### Read from stdin

```bash
cat template.txt | pass-cli inject
```

Or with heredoc:
```bash
pass-cli inject << EOF
{
  "database": {
    "password": "{{ pass://Production/Database/password }}"
  }
}
EOF
```
