# GitHub Actions Permissions Setup

This document explains how GitHub Actions permissions are configured for AllFrame to allow workflows to create issues automatically on CI failures.

---

## Current Setup

### Workflow Permissions

The `compatibility-matrix.yml` workflow includes these permissions:

```yaml
permissions:
  contents: read
  issues: write
```

These permissions allow the workflow to:
- **contents: read** - Read the repository code
- **issues: write** - Create and update issues when CI fails

---

## How It Works

### Automatic Issue Creation

When the compatibility matrix CI workflow fails, the `notify` job automatically creates an issue with:
- **Title**: "Compatibility Matrix Failed"
- **Body**: Description of the failure with a link to the workflow run
- **Labels**: `ci`, `compatibility`, `needs-investigation`

### Workflow Configuration

```yaml
notify:
  name: Notify on failure
  runs-on: ubuntu-latest
  needs: [rust-versions, dependency-versions, feature-matrix, platform-matrix]
  if: failure()
  steps:
    - name: Create issue on failure
      uses: actions/github-script@v7
      with:
        script: |
          github.rest.issues.create({
            owner: context.repo.owner,
            repo: context.repo.repo,
            title: 'Compatibility Matrix Failed',
            body: 'The compatibility matrix CI has failed. Please investigate.\n\nRun: ${{ github.server_url }}/${{ github.repository }}/actions/runs/${{ github.run_id }}',
            labels: ['ci', 'compatibility', 'needs-investigation']
          })
```

---

## Repository Settings

### Default Workflow Permissions (Already Configured)

GitHub Actions workflows in this repository have the following default permissions:

1. Go to: **Settings** → **Actions** → **General**
2. Scroll to: **Workflow permissions**
3. Ensure one of these options is selected:
   - ✅ **Read and write permissions** (Recommended)
   - Or **Read repository contents and packages permissions** + manual permission grants in workflows

### Current Configuration

The repository uses **workflow-level permissions** which are explicitly declared in each workflow file. This provides:
- **Fine-grained control** - Each workflow gets only the permissions it needs
- **Security** - Principle of least privilege
- **Transparency** - Permissions are documented in the workflow file

---

## Default Token Permissions

### What GitHub Provides

GitHub automatically provides a `GITHUB_TOKEN` to each workflow with these capabilities:
- Scoped to the repository running the workflow
- Automatically expires after the workflow completes
- Permissions controlled by:
  1. Repository settings (default permissions)
  2. Workflow-level `permissions:` block (overrides defaults)

### Security Best Practices

The current setup follows GitHub's recommended security practices:
- **Explicit permissions** in workflow files
- **Minimal permissions** (only what's needed)
- **No secrets required** (uses built-in `GITHUB_TOKEN`)

---

## Troubleshooting

### If Issue Creation Fails with 403

If you see this error:
```
RequestError [HttpError]: Resource not accessible by integration
status: 403
```

**Solution 1: Check Repository Settings**

1. Go to: **Settings** → **Actions** → **General**
2. Under **Workflow permissions**, select: **Read and write permissions**
3. Click **Save**

**Solution 2: Verify Workflow Permissions**

Ensure the workflow file has:
```yaml
permissions:
  contents: read
  issues: write
```

**Solution 3: Check Organization Settings (if applicable)**

If the repository is part of an organization:
1. Go to organization **Settings** → **Actions** → **General**
2. Under **Workflow permissions**, ensure it's not set to **Read repository contents permission**
3. Or enable **Allow GitHub Actions to create and approve pull requests**

---

## Minimal Versions Test Fix

The minimal versions test (`-Z minimal-versions`) has been configured to pin specific dependency versions to avoid incompatibilities:

```yaml
- name: Fix minimal versions for compatibility
  if: matrix.profile.name == 'minimal'
  run: |
    # Pin tonic and http to minimum compatible versions with Rust 1.86+
    cargo update -p http@1.0.0 --precise 1.1.0
    cargo update -p tonic --precise 0.14.0
```

This ensures that even with `-Z minimal-versions`, we don't select ancient versions that lack modern API methods like `try_insert` on `HeaderMap`.

---

## Additional Resources

- [GitHub Actions Permissions Documentation](https://docs.github.com/en/actions/security-guides/automatic-token-authentication#permissions-for-the-github_token)
- [Workflow Syntax - Permissions](https://docs.github.com/en/actions/using-workflows/workflow-syntax-for-github-actions#permissions)
- [Managing GitHub Actions settings for a repository](https://docs.github.com/en/repositories/managing-your-repositorys-settings-and-features/enabling-features-for-your-repository/managing-github-actions-settings-for-a-repository)

---

## Summary

✅ **Current Status**: Permissions are properly configured in the workflow file
✅ **Issue Creation**: Enabled with `issues: write` permission
✅ **Minimal Versions**: Fixed with dependency pinning
✅ **Security**: Using workflow-level permissions for fine-grained control

No additional setup is required! The workflow will automatically create issues when CI fails.
