Installers bundled into the ONYKS Chalcedon setup (installed silently when missing):

  svn.msi        TortoiseSVN (includes the svn command line client)
  tailscale.msi  Tailscale for Windows
  psqlodbc_x64.msi  PostgreSQL ODBC driver (64-bit), used by Altium to read the database

The .msi files are large and are not kept in git. Fetch them before `pnpm tauri build`:

  powershell -ExecutionPolicy Bypass -File scripts/fetch-redist.ps1
