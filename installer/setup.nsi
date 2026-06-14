; 软件管理器 — NSIS 安装脚本
; 用法: makensis setup.nsi

!include "MUI2.nsh"

Name "软件管理器"
OutFile "software-manager-setup.exe"
InstallDir "$PROGRAMFILES\SoftwareManager"
RequestExecutionLevel admin

!define MUI_ABORTWARNING

; ----- 页面 -----
!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_LICENSE "..\LICENSE"
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH

!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES

!insertmacro MUI_LANGUAGE "SimpChinese"

; ----- 安装 -----
Section "安装" SecInstall
    SetOutPath "$INSTDIR"

    ; 主程序
    File "..\src-tauri\target\release\software-manager.exe"

    ; 创建快捷方式
    CreateDirectory "$SMPROGRAMS\软件管理器"
    CreateShortCut "$SMPROGRAMS\软件管理器\软件管理器.lnk" "$INSTDIR\software-manager.exe"
    CreateShortCut "$SMPROGRAMS\软件管理器\卸载.lnk" "$INSTDIR\uninstall.exe"
    CreateShortCut "$DESKTOP\软件管理器.lnk" "$INSTDIR\software-manager.exe"

    ; 写入卸载信息
    WriteUninstaller "$INSTDIR\uninstall.exe"
    WriteRegStr HKLM "SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\SoftwareManager" \
        "DisplayName" "软件管理器"
    WriteRegStr HKLM "SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\SoftwareManager" \
        "DisplayVersion" "${VERSION}"
    WriteRegStr HKLM "SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\SoftwareManager" \
        "Publisher" "Alin2077"
    WriteRegStr HKLM "SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\SoftwareManager" \
        "UninstallString" "$INSTDIR\uninstall.exe"
    WriteRegStr HKLM "SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\SoftwareManager" \
        "InstallLocation" "$INSTDIR"
    WriteRegDWORD HKLM "SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\SoftwareManager" \
        "NoModify" 1
    WriteRegDWORD HKLM "SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\SoftwareManager" \
        "NoRepair" 1
SectionEnd

; ----- 卸载 -----
Section "Uninstall"
    Delete "$INSTDIR\software-manager.exe"
    Delete "$INSTDIR\uninstall.exe"
    RMDir "$INSTDIR"

    Delete "$SMPROGRAMS\软件管理器\软件管理器.lnk"
    Delete "$SMPROGRAMS\软件管理器\卸载.lnk"
    RMDir "$SMPROGRAMS\软件管理器"
    Delete "$DESKTOP\软件管理器.lnk"

    DeleteRegKey HKLM "SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\SoftwareManager"
SectionEnd
