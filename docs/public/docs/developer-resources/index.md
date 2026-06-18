# Developer resources

This section lists integrations and tooling for developers who want to build on top of the Proton Pass CLI.

## Ansible integration

Fetch your secrets from Proton Pass and expose them as Ansible variables during playbook execution.

Uses the `pass://` URI scheme together with a field selector to retrieve individual fields from items.

Supports both item-level access and selective field retrieval.

[View repository](https://github.com/protonpass/proton-pass-ansible-integration)

## GitHub Actions

### Install CLI

Installs `pass-cli` into the workflow `PATH` and optionally authenticates via `PROTON_PASS_PERSONAL_ACCESS_TOKEN`.

Useful when you need direct `pass-cli` invocations in your workflow.

```yaml
- uses: protonpass/install-cli-action@v1
  with:
    version: '2.1.4'
  env:
    PROTON_PASS_PERSONAL_ACCESS_TOKEN: ${{ secrets.PROTON_PAT }}
```

[View repository](https://github.com/protonpass/install-cli-action)

### Load secret

Installs `pass-cli`, resolves `pass://` references from environment variables, and exposes them as masked step outputs (or optionally as environment variables).

```yaml
- id: secrets
  uses: protonpass/load-secret-action@v1
  env:
    PROTON_PASS_PERSONAL_ACCESS_TOKEN: ${{ secrets.PROTON_PAT }}
    DB_PASSWORD: pass://MyVault/Database/password

- run: deploy --password "${{ steps.secrets.outputs.DB_PASSWORD }}"
```

[View repository](https://github.com/protonpass/load-secret-action)
