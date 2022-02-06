# Release Checklist

- Ensure local `main` is up to date with `origin/main`.
- Run `cargo update` and review dependency updates.
- Review each crate for changes since last release. If a change has occurred then issue a new release for the crate.
- Update the `CHANGELOG`.
- Update `Cargo.toml` for each crate to new version. Run `cargo update` so that `Cargo.lock` is updated.
- Commit the changes above.
- Push changes to GitHub (w/o tag).
- Wait for `main` CI to complete then push the version tag.
- Wait for `release` CI action to finish creating the release. If the release fails, then delete the tag (locally and origin), delete the release, make fixes and then re-run release.
- Copy relevant section of `CHANGELOG` to the release page.
- Add Unreleased section to the top of the `CHANGELOG`:
```
Unreleased
==========

Unreleased changes. Release notes have not yet been written.
```

