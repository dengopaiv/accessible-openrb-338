; The Windows installer of OpenRB-338 (task-111, task-237). Built by scripts/package-installer.sh,
; which passes the defines below; docs/building.md ("The Windows installer") describes it.
;
;   VERSION   YYYY.MMDD.HHMM, the build time (scripts/version.sh)
;   ARCH      x64 or arm64 (names the file, the title and the registry entry; the binaries
;             are those of the target the script was given)
;   BINDIR    the folder with OpenRB-338.exe and orb.exe (dist/<target>/bin)
;   STAGEDIR  the folder with LICENSE.txt and README.txt (CRLF)
;   PATHPS1   scripts/installer/path.ps1
;   PLUGINDIR the folder with OpenRB-338.vst3 (a folder)
;   ICON      assets/app_icon.ico (the icon of the installer and the uninstaller)
;   OUTFILE   the installer to write
;
; All users only (the user's decision, 2026-11-17: the plugins belong in the machine-wide plugin
; folders, which need administrator rights, so the whole install is per machine): the manifest
; says requireAdministrator, so there is one UAC prompt when the installer or the uninstaller
; starts, and none of the per-user layout of the versions before task-237 is used. The program
; goes to $PROGRAMFILES64\OpenRB-338, the shortcuts to the common Start menu and desktop, the
; uninstall entry and the install record to HKLM (64-bit view), the optional PATH change to the
; machine's PATH, OpenRB-338.vst3 to $COMMONFILES64\VST3 (an optional component, on by default).
; The uninstaller removes what the record in HKLM names (InstallDir, Vst3Dir): the bundle only,
; never the shared plugin folder. A reinstall replaces the bundle.
;
; CLAP was dropped (task-278, the user's decision of 2026-11-22). The releases up to
; v2026.1118.0919 installed OpenRB-338.clap to $COMMONFILES64\CLAP (a component "CLAP plugin",
; the record ClapDir): the installer deletes that one file (and the one in ClapDir of the record),
; whether or not the VST3 component is picked, and so does the uninstaller (RemoveOldClap).
; Nothing else in a CLAP folder is touched.
;
; An earlier per-user install (HKCU, %LOCALAPPDATA%\Programs\OpenRB-338) is left as it is: the
; installer tells the user it is there and how to remove it (Settings > Apps).
;
; /D=DIR installs the program elsewhere, /S runs silently. Silent options (the checkboxes, for
; /S): /STARTMENU=0|1 (default 1), /DESKTOP=0|1 (default 0), /PATH=0|1 (default 0) and
; /VST3=0|1 (default 1); /VST3DIR=DIR (quoted when it holds a space) chooses another plugin
; folder. The options /CLAP= and /CLAPDIR= of the earlier installers are ignored. Example:
;   setup.exe /S /PATH=1 /VST3=0 /D=C:\Tools\OpenRB-338

Unicode true
SetCompressor /SOLID lzma
RequestExecutionLevel admin
ManifestDPIAware true

!ifndef VERSION | ARCH | BINDIR | STAGEDIR | PATHPS1 | PLUGINDIR | ICON | OUTFILE
  !error "VERSION, ARCH, BINDIR, STAGEDIR, PATHPS1, PLUGINDIR, ICON and OUTFILE must be defined (-D)"
!endif

!define NAME "OpenRB-338"
!define UNINST_KEY "Software\Microsoft\Windows\CurrentVersion\Uninstall\${NAME}"
!define APP_KEY "Software\${NAME}"
!define VST3_NAME "${NAME}.vst3"
!define CLAP_NAME "${NAME}.clap"

; the plugin folder (set by .onInit; /VST3DIR= changes it)
Var Vst3Dir

Name "${NAME}"
OutFile "${OUTFILE}"
InstallDir "$PROGRAMFILES64\${NAME}"
InstallDirRegKey HKLM "${APP_KEY}" "InstallDir"
BrandingText "${NAME} ${VERSION} (${ARCH})"

VIProductVersion "${VERSION}.0"
VIAddVersionKey "ProductName" "${NAME}"
VIAddVersionKey "FileDescription" "${NAME} installer (${ARCH})"
VIAddVersionKey "FileVersion" "${VERSION}"
VIAddVersionKey "ProductVersion" "${VERSION}"
VIAddVersionKey "LegalCopyright" "GPL-3.0-only"

!include "MUI2.nsh"
!include "FileFunc.nsh"
!include "LogicLib.nsh"
!include "Sections.nsh"
!include "WinMessages.nsh"
!insertmacro GetParameters
!insertmacro GetOptions
!insertmacro GetSize

!define MUI_ICON "${ICON}"
!define MUI_UNICON "${ICON}"
!define MUI_ABORTWARNING

!define MUI_WELCOMEPAGE_TEXT "This installs ${NAME} ${VERSION} (${ARCH}), a reimplementation of the audio engine of Propellerhead ReBirth RB-338 2.01, and its command line tool orb, for all users of this computer, with the optional VST3 plugin.$\r$\n$\r$\nIt needs no file of the original program: the standard drum kit is built in.$\r$\n$\r$\nThe installer is not signed, which is why Windows asks whether to run it. It needs administrator rights (the plugin folders are shared by all users), so Windows asks for your permission when it starts."
!insertmacro MUI_PAGE_WELCOME
!define MUI_LICENSEPAGE_TEXT_TOP "${NAME} is free software under the GNU General Public License, version 3 only."
!insertmacro MUI_PAGE_LICENSE "${STAGEDIR}\LICENSE.txt"
!insertmacro MUI_PAGE_COMPONENTS
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!define MUI_FINISHPAGE_RUN "$INSTDIR\${NAME}.exe"
!define MUI_FINISHPAGE_RUN_TEXT "Start ${NAME}"
!define MUI_FINISHPAGE_SHOWREADME "$INSTDIR\README.txt"
!define MUI_FINISHPAGE_SHOWREADME_TEXT "Show the readme"
!insertmacro MUI_PAGE_FINISH

!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES
!insertmacro MUI_UNPAGE_FINISH

!insertmacro MUI_LANGUAGE "English"

; ---------------------------------------------------------------------------------------------

; A running OpenRB-338.exe or orb.exe cannot be overwritten or deleted. Opening the file for
; writing fails while a program runs from it (a sharing violation), so that is the check; it
; changes nothing. EnsureClosed asks the user to close the program (Retry); under /S, or on
; Cancel, it sets the exit code 5 and aborts before anything is changed.
!macro LockFunctions UN
  Function ${UN}CheckLocked
    StrCpy $0 0
    ${ForEach} $2 1 2 + 1
      ${If} $2 == 1
        StrCpy $3 "$INSTDIR\${NAME}.exe"
      ${Else}
        StrCpy $3 "$INSTDIR\orb.exe"
      ${EndIf}
      ${If} ${FileExists} $3
        ClearErrors
        FileOpen $4 $3 a
        ${If} ${Errors}
          StrCpy $0 1
        ${Else}
          FileClose $4
        ${EndIf}
      ${EndIf}
    ${Next}
  FunctionEnd

  Function ${UN}EnsureClosed
    retry:
    Call ${UN}CheckLocked
    ${If} $0 == 1
      MessageBox MB_RETRYCANCEL|MB_ICONEXCLAMATION "${NAME} or orb is running from $INSTDIR.$\r$\nClose it, then choose Retry." /SD IDCANCEL IDRETRY retry
      SetErrorLevel 5
      Abort "${NAME} or orb is running from $INSTDIR: close it and run this again (exit code 5)."
    ${EndIf}
  FunctionEnd
!macroend
!insertmacro LockFunctions ""
!insertmacro LockFunctions "un."

; The CLAP plugin of the earlier releases (task-278): the file OpenRB-338.clap in the common CLAP
; folder and in the folder of the record's ClapDir, and nothing else. The record's value goes.
; DetailPrint says what was found.
!macro RemoveOldClap PFX
  Function ${PFX}RemoveOldClap
    ReadRegStr $2 HKLM "${APP_KEY}" "ClapDir"
    ${If} $2 != ""
    ${AndIf} ${FileExists} "$2\${CLAP_NAME}"
      Delete "$2\${CLAP_NAME}"
      ${If} ${FileExists} "$2\${CLAP_NAME}"
        DetailPrint "The old $2\${CLAP_NAME} could not be removed (a host has it open?). Delete it by hand."
      ${Else}
        DetailPrint "Removed the old $2\${CLAP_NAME} (the CLAP plugin was dropped)."
      ${EndIf}
    ${EndIf}
    ${If} ${FileExists} "$COMMONFILES64\CLAP\${CLAP_NAME}"
      Delete "$COMMONFILES64\CLAP\${CLAP_NAME}"
      ${If} ${FileExists} "$COMMONFILES64\CLAP\${CLAP_NAME}"
        DetailPrint "The old $COMMONFILES64\CLAP\${CLAP_NAME} could not be removed (a host has it open?). Delete it by hand."
      ${Else}
        DetailPrint "Removed the old $COMMONFILES64\CLAP\${CLAP_NAME} (the CLAP plugin was dropped)."
      ${EndIf}
    ${EndIf}
    DeleteRegValue HKLM "${APP_KEY}" "ClapDir"
  FunctionEnd
!macroend
!insertmacro RemoveOldClap ""
!insertmacro RemoveOldClap "un."

Section "${NAME} and orb" SecApp
  SectionIn RO
  Call EnsureClosed
  ; the old CLAP plugin goes whatever components are picked
  Call RemoveOldClap
  ; An install before task-237 was per user (HKCU, %LOCALAPPDATA%\Programs\OpenRB-338). It is
  ; not touched (the HKCU of an elevated process is not always the user's, and its files are not
  ; ours to delete unasked): the user is told, and how to remove it.
  ReadRegStr $2 HKCU "${APP_KEY}" "InstallDir"
  ${If} $2 != ""
    DetailPrint "An earlier per-user install of ${NAME} is in $2. It is left as it is."
    MessageBox MB_OK|MB_ICONINFORMATION "An earlier per-user install of ${NAME} is in $2.$\r$\nIt stays, and so do its PATH entry and shortcuts, until you remove it: Settings > Apps > Installed apps > ${NAME} (the entry of that install) > Uninstall. This install is for all users." /SD IDOK
  ${EndIf}
  ; An install over an earlier one keeps its folder, unless /D= or the folder page chose
  ; another: then the PATH entry of the earlier folder (if there is one) would point nowhere,
  ; so it goes. The files of the earlier folder stay.
  ReadRegStr $2 HKLM "${APP_KEY}" "InstallDir"
  ${If} $2 != ""
  ${AndIf} $2 != "$INSTDIR"
    InitPluginsDir
    File "/oname=$PLUGINSDIR\path.ps1" "${PATHPS1}"
    nsExec::ExecToLog 'powershell.exe -NoProfile -NonInteractive -ExecutionPolicy Bypass -File "$PLUGINSDIR\path.ps1" remove "$2" machine'
    Pop $0
    SendMessage ${HWND_BROADCAST} ${WM_WININICHANGE} 0 "STR:Environment" /TIMEOUT=5000
  ${EndIf}
  SetOutPath "$INSTDIR"
  File "${BINDIR}\${NAME}.exe"
  File "${BINDIR}\orb.exe"
  File "${STAGEDIR}\LICENSE.txt"
  File "${STAGEDIR}\README.txt"
  !ifdef NOTICES
    File "${NOTICES}"
  !endif
  WriteUninstaller "$INSTDIR\Uninstall.exe"

  WriteRegStr HKLM "${APP_KEY}" "InstallDir" "$INSTDIR"
  WriteRegStr HKLM "${APP_KEY}" "Version" "${VERSION}"

  ; Settings > Apps > Installed apps (Add or remove programs)
  WriteRegStr HKLM "${UNINST_KEY}" "DisplayName" "${NAME}"
  WriteRegStr HKLM "${UNINST_KEY}" "DisplayVersion" "${VERSION}"
  WriteRegStr HKLM "${UNINST_KEY}" "Publisher" "${NAME}"
  WriteRegStr HKLM "${UNINST_KEY}" "DisplayIcon" "$INSTDIR\${NAME}.exe"
  WriteRegStr HKLM "${UNINST_KEY}" "InstallLocation" "$INSTDIR"
  WriteRegStr HKLM "${UNINST_KEY}" "UninstallString" '"$INSTDIR\Uninstall.exe"'
  WriteRegStr HKLM "${UNINST_KEY}" "QuietUninstallString" '"$INSTDIR\Uninstall.exe" /S'
  WriteRegDWORD HKLM "${UNINST_KEY}" "NoModify" 1
  WriteRegDWORD HKLM "${UNINST_KEY}" "NoRepair" 1
  ${GetSize} "$INSTDIR" "/S=0K" $0 $1 $2
  WriteRegDWORD HKLM "${UNINST_KEY}" "EstimatedSize" $0
SectionEnd

Section "Start menu shortcut" SecStartMenu
  CreateShortcut "$SMPROGRAMS\${NAME}.lnk" "$INSTDIR\${NAME}.exe"
SectionEnd

Section /o "Desktop shortcut" SecDesktop
  CreateShortcut "$DESKTOP\${NAME}.lnk" "$INSTDIR\${NAME}.exe"
SectionEnd

; The bundles are the program's own names: an earlier copy goes first (a folder that is not
; ours is never touched), so a reinstall or an upgrade leaves no stale file. VST3: a host that
; holds the old folder open keeps it, and the installer says so (a message, then it aborts).
Section "VST3 plugin" SecVst3
  ; the folder of an earlier install is not ours any more when another one was chosen
  ReadRegStr $2 HKLM "${APP_KEY}" "Vst3Dir"
  ${If} $2 != ""
  ${AndIf} $2 != $Vst3Dir
    RMDir /r "$2\${VST3_NAME}"
  ${EndIf}
  CreateDirectory "$Vst3Dir"
  RMDir /r "$Vst3Dir\${VST3_NAME}"
  ${If} ${FileExists} "$Vst3Dir\${VST3_NAME}"
    MessageBox MB_OK|MB_ICONEXCLAMATION "The old $Vst3Dir\${VST3_NAME} could not be removed: a host may have it open. Close the host and install again." /SD IDOK
    Abort "The old ${VST3_NAME} could not be removed."
  ${EndIf}
  SetOutPath "$Vst3Dir\${VST3_NAME}"
  File /r "${PLUGINDIR}\${VST3_NAME}\*.*"
  WriteRegStr HKLM "${APP_KEY}" "Vst3Dir" "$Vst3Dir"
SectionEnd

Section /o "Add orb to the PATH" SecPath
  SetOutPath "$INSTDIR"
  File "/oname=$INSTDIR\path.ps1" "${PATHPS1}"
  nsExec::ExecToLog 'powershell.exe -NoProfile -NonInteractive -ExecutionPolicy Bypass -File "$INSTDIR\path.ps1" add "$INSTDIR" machine'
  Pop $0
  ${If} $0 == 0
    ; tell running programs (Explorer, so new terminals) that the environment changed
    SendMessage ${HWND_BROADCAST} ${WM_WININICHANGE} 0 "STR:Environment" /TIMEOUT=5000
  ${Else}
    DetailPrint "Could not change the PATH (exit code $0). Add $INSTDIR to it by hand."
    MessageBox MB_OK|MB_ICONEXCLAMATION "The PATH could not be changed (exit code $0).$\r$\nAdd $INSTDIR to the PATH by hand to run orb from any folder." /SD IDOK
  ${EndIf}
SectionEnd

!insertmacro MUI_FUNCTION_DESCRIPTION_BEGIN
  !insertmacro MUI_DESCRIPTION_TEXT ${SecApp} "The app ${NAME}.exe and the command line tool orb.exe."
  !insertmacro MUI_DESCRIPTION_TEXT ${SecStartMenu} "A shortcut to ${NAME} in the Start menu of all users."
  !insertmacro MUI_DESCRIPTION_TEXT ${SecDesktop} "A shortcut to ${NAME} on the desktop of all users."
  !insertmacro MUI_DESCRIPTION_TEXT ${SecVst3} "The VST3 instrument ${VST3_NAME}, in the common VST3 folder (Program Files\Common Files\VST3). Rescan plugins in your host."
  !insertmacro MUI_DESCRIPTION_TEXT ${SecPath} "Adds the install folder to the PATH of all users, so orb works in any new terminal."
!insertmacro MUI_FUNCTION_DESCRIPTION_END

; /STARTMENU=, /DESKTOP=, /PATH= and /VST3= set the checkboxes (for /S too)
!macro OptionSection NAME SEC
  ClearErrors
  ${GetOptions} $0 "/${NAME}=" $1
  ${IfNot} ${Errors}
    ${If} $1 == "1"
      !insertmacro SelectSection ${SEC}
    ${Else}
      !insertmacro UnselectSection ${SEC}
    ${EndIf}
  ${EndIf}
!macroend

Function .onInit
  ; the 64-bit registry view and folders, also for the 32-bit installer stub on Windows on Arm
  SetRegView 64
  ; the Start menu and desktop of all users
  SetShellVarContext all
  ${GetParameters} $0
  !insertmacro OptionSection "STARTMENU" ${SecStartMenu}
  !insertmacro OptionSection "DESKTOP" ${SecDesktop}
  !insertmacro OptionSection "PATH" ${SecPath}
  !insertmacro OptionSection "VST3" ${SecVst3}
  StrCpy $Vst3Dir "$COMMONFILES64\VST3"
  ClearErrors
  ${GetOptions} $0 "/VST3DIR=" $1
  ${IfNot} ${Errors}
  ${AndIf} $1 != ""
    StrCpy $Vst3Dir $1
  ${EndIf}
  ClearErrors
FunctionEnd

; ---------------------------------------------------------------------------------------------

Function un.onInit
  SetRegView 64
  SetShellVarContext all
FunctionEnd

Section "Uninstall"
  ; nothing is changed while the program runs (exit code 5 under /S)
  Call un.EnsureClosed
  ; the PATH entry goes first, while path.ps1 is still there (nothing happens without an entry)
  ${If} ${FileExists} "$INSTDIR\path.ps1"
    nsExec::ExecToLog 'powershell.exe -NoProfile -NonInteractive -ExecutionPolicy Bypass -File "$INSTDIR\path.ps1" remove "$INSTDIR" machine'
    Pop $0
    SendMessage ${HWND_BROADCAST} ${WM_WININICHANGE} 0 "STR:Environment" /TIMEOUT=5000
  ${EndIf}

  ; the plugin, from the folder the installer wrote down in HKLM; only our name (the shared
  ; folder stays)
  ReadRegStr $2 HKLM "${APP_KEY}" "Vst3Dir"
  ${If} $2 != ""
    RMDir /r "$2\${VST3_NAME}"
    ${If} ${FileExists} "$2\${VST3_NAME}"
      DetailPrint "$2\${VST3_NAME} could not be removed (a host has it open?). Delete it by hand."
    ${EndIf}
  ${EndIf}
  ; an old CLAP plugin of an earlier release, if one is still there
  Call un.RemoveOldClap

  Delete "$SMPROGRAMS\${NAME}.lnk"
  Delete "$DESKTOP\${NAME}.lnk"
  Delete "$INSTDIR\${NAME}.exe"
  Delete "$INSTDIR\orb.exe"
  Delete "$INSTDIR\LICENSE.txt"
  Delete "$INSTDIR\README.txt"
  Delete "$INSTDIR\THIRD-PARTY-NOTICES.txt"
  Delete "$INSTDIR\path.ps1"
  Delete "$INSTDIR\Uninstall.exe"
  ; not recursive: a folder the user put files in stays
  RMDir "$INSTDIR"
  ; a file that could not be deleted keeps the entry in Apps, so the uninstall can be repeated
  ${If} ${FileExists} "$INSTDIR\${NAME}.exe"
  ${OrIf} ${FileExists} "$INSTDIR\orb.exe"
    SetErrorLevel 6
    MessageBox MB_OK|MB_ICONEXCLAMATION "Some files in $INSTDIR could not be deleted. The uninstall entry stays: close what uses them and uninstall again." /SD IDOK
    Abort "Files could not be deleted (exit code 6)."
  ${EndIf}
  DeleteRegKey HKLM "${UNINST_KEY}"
  DeleteRegKey HKLM "${APP_KEY}"
SectionEnd
