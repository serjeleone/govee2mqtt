# Publishing a prepared release

This fork uses `YYYY-MM-DD` for the app version, GitHub release title and tag.

Commit the app version in `addon/config.yaml`, its changelog, and release notes at `.github/release-notes/YYYY-MM-DD.md` to `main` together with the tested source. A change under that notes directory is an explicit publication request; ordinary source pushes do not publish.

The existing Verify, build and publish workflow validates metadata, runs Clippy, compiles, and runs unit, LAN simulator and MQTT integration tests. Only after those checks succeed does it create the lightweight date tag at that immutable source SHA and publish a stable latest GitHub release using the prepared notes. It then builds standalone amd64/aarch64 images, publishes their manifest, and builds/signs the Home Assistant images pinned to that release.

The workflow continues publication in the same run because releases created with `GITHUB_TOKEN` do not trigger another release-event workflow. Manual release publication and nonpublishing workflow_dispatch remain supported. Credentials are handled by GitHub Actions; no personal access token is required in source.

Retries can resume a matching tag/release at the same SHA. A tag pointing to any other object is rejected and never replaced. Failed build jobs can be rerun without moving the tag.
