# ClawGuard npm Wrapper

This package installs the platform-specific `clawguard` binary from GitHub Releases and exposes it as an npm executable.
The downloaded archive is verified against the signed release manifest before extraction.

After publishing, users will be able to run:

```bash
npx clawguard --help
```
