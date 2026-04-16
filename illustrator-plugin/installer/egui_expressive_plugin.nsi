; egui_expressive Illustrator Plugin Installer
; Requires NSIS (https://nsis.sourceforge.io/)

!define PRODUCT_NAME "egui_expressive Illustrator Plugin"
!define PRODUCT_VERSION "1.0.0"
!define PLUGIN_FOLDER_NAME "egui_expressive_export"

Name "${PRODUCT_NAME} ${PRODUCT_VERSION}"
OutFile "egui_expressive_plugin_installer.exe"
InstallDir "$APPDATA\Adobe\UXP\PluginsStorage\ILST\28\develop\${PLUGIN_FOLDER_NAME}"
RequestExecutionLevel user

Page directory
Page instfiles

Section "Plugin Files"
    SetOutPath "$INSTDIR"
    File "..\manifest.json"
    File "..\plugin.js"
    File "..\index.html"

    ; Write uninstaller
    WriteUninstaller "$INSTDIR\uninstall.exe"

    ; Write registry entry
    WriteRegStr HKCU "Software\egui_expressive\IllustratorPlugin" "InstallDir" "$INSTDIR"

    MessageBox MB_OK "Plugin installed successfully!$\n$\nPlease restart Adobe Illustrator and enable the plugin in:$\nPlugins > Plugin Manager"
SectionEnd

Section "Uninstall"
    Delete "$INSTDIR\manifest.json"
    Delete "$INSTDIR\plugin.js"
    Delete "$INSTDIR\index.html"
    Delete "$INSTDIR\uninstall.exe"
    RMDir "$INSTDIR"
    DeleteRegKey HKCU "Software\egui_expressive\IllustratorPlugin"
SectionEnd
