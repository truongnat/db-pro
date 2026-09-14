DB Pro 0.1.0 - portable build
=============================

This archive contains a portable, unsigned build of the DB Pro desktop
application (native binary). No installer is provided for v0.1.

Contents by platform
--------------------
macOS   : "DB Pro.app" - run it from this folder or drag it to Applications.
Windows : db-pro-native.exe
Linux   : db-pro-native

Running it
----------
macOS   : open "DB Pro.app". The build is unsigned and not notarized, so
          Gatekeeper warns on first launch: right-click the app, choose
          "Open", then confirm. This build is for Apple Silicon (arm64) only.
Windows : run db-pro-native.exe. The build is unsigned, so SmartScreen may
          warn on first launch ("More info" > "Run anyway").
Linux   : make the file executable if needed (chmod +x db-pro-native) and run
          it. Requires an X11 or Wayland session with a working OpenGL driver,
          and a D-Bus Secret Service provider (for example gnome-keyring) to
          store connection secrets.

First run
---------
DB Pro keeps its workspace state (connections, saved queries, query history,
open tabs, settings) in a ".db-pro-data" directory next to the directory the
app is started from. Set the DB_PRO_DATA_DIR environment variable to choose a
different location.

Optional external tools
-----------------------
PostgreSQL backup and restore shell out to "pg_dump" and "pg_restore", and SSH
tunnels use the system "ssh" client. Install them separately if you need those
features.

Character encoding
------------------
The bundled interface font uses the SIL Open Font License; the text of that
license ships inside the application binary. The DB Pro project itself has no
public license yet, so this build is not licensed for redistribution.

Verification
------------
SHA256SUMS.txt in the release lists the SHA-256 checksum of this archive.
Verify the download before running it:

  macOS/Linux : shasum -a 256 <archive>        (or: sha256sum <archive>)
  Windows     : Get-FileHash <archive> -Algorithm SHA256
