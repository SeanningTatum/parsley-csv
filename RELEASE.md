# Release Guide

This document explains how to create releases for `table-to-csv`.

## 🚀 Quick Start - Automatic Releases

The easiest way to create a release:

1. **Update version in `Cargo.toml`:**
   ```toml
   version = "0.4.1"  # Change this
   ```

2. **Commit and push to main:**
   ```bash
   git add Cargo.toml
   git commit -m "Bump version to 0.4.1"
   git push origin main
   ```

3. **That's it!** 🎉

GitHub Actions will automatically:
- ✅ Detect the version change
- ✅ Create a git tag `v0.4.1`
- ✅ Build binaries for all platforms:
  - macOS Intel (x86_64)
  - macOS Apple Silicon (ARM64)
  - Linux x86_64
  - Linux ARM64
  - Windows x86_64
- ✅ Create a GitHub Release with all binaries attached
- ✅ Generate SHA256 checksums

## 📋 Release Methods

### Method 1: Automatic (Recommended)
**When:** Push version change to main branch  
**How:** Update `Cargo.toml` version and push  
**Result:** Automatic tag creation and release

### Method 2: Manual Tag from CLI
**When:** You want manual control  
**How:**
```bash
git tag v0.4.1
git push origin v0.4.1
```
**Result:** Workflow triggers on tag push

### Method 3: GitHub Dashboard
**When:** Creating a release from the web interface  
**How:**
1. Go to repository → "Releases" → "Create a new release"
2. Click "Choose a tag" → Enter `v0.4.1` → "Create new tag"
3. Click "Publish release"

**Result:** Workflow triggers and adds binaries to the release

### Method 4: Manual Workflow Dispatch
**When:** Testing or re-running a build  
**How:**
1. Go to "Actions" tab → "Release" workflow
2. Click "Run workflow" → Select branch
3. Click "Run workflow"

**Result:** Builds and releases using current version in `Cargo.toml`

## 🏗️ Local Build Only

If you just want to build binaries locally without releasing:

```bash
./build-release.sh
```

This creates distribution files in `dist/` directory.

## 📦 What Gets Built

Each release includes:

### Archives
- `table-to-csv-macos-x86_64.tar.gz` - Intel Macs
- `table-to-csv-macos-aarch64.tar.gz` - Apple Silicon Macs
- `table-to-csv-linux-x86_64.tar.gz` - Linux Intel/AMD
- `table-to-csv-linux-aarch64.tar.gz` - Linux ARM64
- `table-to-csv-windows-x86_64.zip` - Windows 64-bit

### Checksums
- `SHA256SUMS` - Verification checksums for all archives

## 🔍 Monitoring Releases

To check if a release is in progress:

1. Go to the "Actions" tab in GitHub
2. Look for the "Release" workflow
3. Click on the running workflow to see progress

The entire process typically takes **5-10 minutes**.

## ⚙️ Workflow Behavior

The workflow intelligently determines when to release:

| Trigger | Cargo.toml Changed? | Version Changed? | Action |
|---------|-------------------|------------------|---------|
| Push to `main` | ✅ Yes | ✅ Yes | 🚀 **Release** |
| Push to `main` | ✅ Yes | ❌ No | ⏭️ Skip |
| Push to `main` | ❌ No | - | ⏭️ Skip |
| Tag push `v*` | - | - | 🚀 **Release** |
| Manual dispatch | - | - | 🚀 **Release** |

## 🛠️ Troubleshooting

### Release didn't trigger after version change
- Ensure `Cargo.toml` was actually modified in the commit
- Check the "Actions" tab for workflow runs
- Verify the version string changed (not just other fields)

### Build failed
- Check the "Actions" tab for error logs
- Common issues:
  - Compilation errors in the code
  - Test failures
  - Network issues downloading dependencies

### Tag already exists
- Delete the tag: `git tag -d v0.4.1 && git push origin :refs/tags/v0.4.1`
- Create it again with the correct commit

## 📝 Version Numbering

Follow [Semantic Versioning](https://semver.org/):

- **MAJOR** (1.0.0): Breaking changes
- **MINOR** (0.5.0): New features, backwards compatible
- **PATCH** (0.4.1): Bug fixes, backwards compatible

## 🎯 Best Practices

1. **Always test locally** before releasing: `cargo test && cargo build --release`
2. **Update CHANGELOG** (if you have one) before bumping version
3. **Use meaningful commit messages** for version bumps
4. **Test the binaries** after release to ensure they work
5. **Keep releases frequent** - small, incremental releases are better than large ones

---

For questions or issues with the release process, check the workflow file:
`.github/workflows/release.yml`

