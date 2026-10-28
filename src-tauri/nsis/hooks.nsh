; Installs the programs ONYKS Chalcedon depends on - the SVN command line client,
; Tailscale and the PostgreSQL ODBC driver (needed by Altium to read the database) -
; from the MSI packages bundled in $INSTDIR\redist (see redist\README.txt).
; Each one is skipped when it is already present on the computer.

!macro NSIS_HOOK_POSTINSTALL
  ; --- Subversion (svn.exe) ---
  nsExec::Exec 'cmd /c "svn --version --quiet >nul 2>&1"'
  Pop $0
  ${If} $0 != 0
    ${If} ${FileExists} "$INSTDIR\redist\svn.msi"
      DetailPrint "Installing Subversion..."
      ; ADDLOCAL=ALL also installs the command line client tools of TortoiseSVN.
      ExecWait '"$SYSDIR\msiexec.exe" /i "$INSTDIR\redist\svn.msi" /qn /norestart ADDLOCAL=ALL' $1
      DetailPrint "Subversion installer exit code: $1"
    ${EndIf}
  ${Else}
    DetailPrint "Subversion is already installed."
  ${EndIf}

  ; --- Tailscale ---
  ${If} ${FileExists} "$PROGRAMFILES64\Tailscale\tailscale.exe"
  ${OrIf} ${FileExists} "$PROGRAMFILES\Tailscale\tailscale.exe"
    DetailPrint "Tailscale is already installed."
  ${Else}
    ${If} ${FileExists} "$INSTDIR\redist\tailscale.msi"
      DetailPrint "Installing Tailscale..."
      ExecWait '"$SYSDIR\msiexec.exe" /i "$INSTDIR\redist\tailscale.msi" /qn /norestart' $1
      DetailPrint "Tailscale installer exit code: $1"
    ${EndIf}
  ${EndIf}

  ; --- PostgreSQL ODBC driver (psqlODBC, 64-bit) ---
  ; The driver registers itself in the 64-bit registry view (Tauri installers run in it).
  SetRegView 64
  ReadRegStr $2 HKLM "SOFTWARE\ODBC\ODBCINST.INI\ODBC Drivers" "PostgreSQL Unicode(x64)"
  ${If} $2 == "Installed"
    DetailPrint "PostgreSQL ODBC driver is already installed."
  ${Else}
    ${If} ${FileExists} "$INSTDIR\redist\psqlodbc_x64.msi"
      DetailPrint "Installing PostgreSQL ODBC driver..."
      ExecWait '"$SYSDIR\msiexec.exe" /i "$INSTDIR\redist\psqlodbc_x64.msi" /qn /norestart' $1
      DetailPrint "PostgreSQL ODBC driver installer exit code: $1"
    ${EndIf}
  ${EndIf}
!macroend
