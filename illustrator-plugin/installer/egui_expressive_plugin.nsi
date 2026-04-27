; egui_expressive Illustrator Plugin Installer
; Requires NSIS (https://nsis.sourceforge.io/)

!define PRODUCT_NAME "egui_expressive Illustrator Plugin"
!define PRODUCT_VERSION "1.0.0"
!define PLUGIN_FOLDER_NAME "egui_expressive_export"

Name "${PRODUCT_NAME} ${PRODUCT_VERSION}"
OutFile "egui_expressive_plugin_installer.exe"
InstallDir "$APPDATA\Adobe\CEP\extensions\com.egui-expressive.illustrator-exporter"
RequestExecutionLevel user

Page directory
Page instfiles

Section "Plugin Files"
    SetOutPath "$INSTDIR\CSXS"
    File "..\CSXS\manifest.xml"
    SetOutPath "$INSTDIR"
    File "..\host.jsx"
    File "..\plugin.js"
    File "..\index.html"
    SetOutPath "$INSTDIR\bin"
    File /r "..\bin\*.*"

    ; Write uninstaller
    WriteUninstaller "$INSTDIR\uninstall.exe"

    ; Write registry entry
    WriteRegStr HKCU "Software\egui_expressive\IllustratorPlugin" "InstallDir" "$INSTDIR"

    MessageBox MB_OK "Plugin installed successfully!$\n$\nPlease restart Adobe Illustrator and enable the plugin in:$\nWindow → Extensions → egui_expressive Export"
SectionEnd

Section "Uninstall"
    Delete "$INSTDIR\CSXS\manifest.xml"
    RMDir "$INSTDIR\CSXS"
    Delete "$INSTDIR\host.jsx"
    Delete "$INSTDIR\plugin.js"
    Delete "$INSTDIR\index.html"
    RMDir /r "$INSTDIR\bin"
    Delete "$INSTDIR\uninstall.exe"
    RMDir "$INSTDIR"
    DeleteRegKey HKCU "Software\egui_expressive\IllustratorPlugin"
SectionEnd
