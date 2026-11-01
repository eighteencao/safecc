Unicode True
RequestExecutionLevel admin
ManifestSupportedOS win10
Name "safecc"
Caption "safecc · Claude 环境安全助手"
OutFile "${OUT_FILE}"
InstallDir "$PROGRAMFILES64\safecc"
InstallDirRegKey HKLM "Software\safecc" "InstallDir"
Icon "${ICON_FILE}"
UninstallIcon "${ICON_FILE}"
ShowInstDetails show
ShowUninstDetails show

VIProductVersion "0.1.0.0"
VIAddVersionKey "ProductName" "safecc"
VIAddVersionKey "CompanyName" "safecc"
VIAddVersionKey "FileDescription" "safecc · Claude 环境安全助手"
VIAddVersionKey "FileVersion" "0.1.0"
VIAddVersionKey "ProductVersion" "0.1.0"
VIAddVersionKey "LegalCopyright" "safecc"

Page directory
Page instfiles
UninstPage uninstConfirm
UninstPage instfiles

Section "safecc" SecMain
  SetOutPath "$INSTDIR"
  File "${APP_ROOT}\claude_environment_assistant.exe"
  File "${APP_ROOT}\WebView2Loader.dll"

  SetOutPath "$INSTDIR\deployment\ccbaga-network"
  File /r "${APP_ROOT}\deployment\ccbaga-network\*"

  CreateDirectory "$SMPROGRAMS\safecc"
  CreateShortCut "$SMPROGRAMS\safecc\safecc.lnk" "$INSTDIR\claude_environment_assistant.exe"
  CreateShortCut "$DESKTOP\safecc.lnk" "$INSTDIR\claude_environment_assistant.exe"

  WriteRegStr HKLM "Software\safecc" "InstallDir" "$INSTDIR"
  WriteUninstaller "$INSTDIR\uninstall.exe"
SectionEnd

Section "Uninstall"
  Delete "$DESKTOP\safecc.lnk"
  Delete "$SMPROGRAMS\safecc\safecc.lnk"
  RMDir "$SMPROGRAMS\safecc"
  DeleteRegKey HKLM "Software\safecc"
  RMDir /r "$INSTDIR"
SectionEnd
