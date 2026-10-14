; EVKey Windows installer (Inno Setup 6).
; Build: scripts/build-installer.ps1 (passes /DAppVersion=<Cargo workspace version>).
; Files go to {autopf}\EVKey\<version>\. Versions up to 2.2.x shipped a TSF DLL that stays locked while apps have it
; loaded, so each release still gets its own folder and older folders are removed best effort (locked files are
; deleted on reboot, the user is never forced to reboot).

#ifndef AppVersion
  #error AppVersion must be passed with /DAppVersion=<version>
#endif
#ifndef BinDir
  #define BinDir "..\target\release"
#endif
#ifndef OutDir
  #define OutDir "..\dist"
#endif

#define AppName "EVKey"
#define Publisher "chrisnguyen"
; CLSID of the TSF text service shipped up to 2.2.x (ADR 0006): unregistered on upgrade.
#define LegacyTsfClsid "{B5F8D54F-6BAD-46D4-8DBB-FFE03BF4D2F4}"

[Setup]
AppId={{0FCB8015-6E36-4DEC-B8B6-7BAC1935539B}
AppName={#AppName}
AppVerName={#AppName} {#AppVersion}
AppVersion={#AppVersion}
AppPublisher={#Publisher}
VersionInfoVersion={#AppVersion}.0
VersionInfoProductVersion={#AppVersion}
UninstallDisplayName={#AppName} {#AppVersion}
DefaultDirName={autopf}\EVKey\{#AppVersion}
UsePreviousAppDir=no
DisableDirPage=yes
DefaultGroupName={#AppName}
DisableProgramGroupPage=yes
PrivilegesRequired=admin
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
LicenseFile=..\LICENSE
OutputDir={#OutDir}
OutputBaseFilename=EVKey-Setup-{#AppVersion}
Compression=lzma2/max
SolidCompression=yes
WizardStyle=modern
; EVKey is stopped by the installer itself (taskkill), so no Restart Manager prompt.
CloseApplications=no
RestartApplications=no
ShowLanguageDialog=auto

[Languages]
; Inno picks the language matching the system UI; English is the fallback (first entry).
Name: "english"; MessagesFile: "compiler:Default.isl"
Name: "vietnamese"; MessagesFile: "languages\Vietnamese.isl"

[CustomMessages]
english.TaskAutostart=Start EVKey with Windows
vietnamese.TaskAutostart=Khởi động EVKey cùng Windows
english.LaunchEVKey=Launch EVKey
vietnamese.LaunchEVKey=Chạy EVKey
english.SettingsShortcut=EVKey
vietnamese.SettingsShortcut=Cài đặt EVKey
english.UninstallKept=Your settings in %APPDATA%\EVKey were kept. Delete that folder yourself if you no longer need them.
vietnamese.UninstallKept=Cấu hình trong %APPDATA%\EVKey đã được giữ lại. Hãy tự xóa thư mục đó nếu không còn cần.

[Tasks]
Name: "autostart"; Description: "{cm:TaskAutostart}"; Flags: checkedonce

[Files]
Source: "{#BinDir}\evkey.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\LICENSE"; DestDir: "{app}"; DestName: "LICENSE.txt"; Flags: ignoreversion

[Icons]
Name: "{group}\{cm:SettingsShortcut}"; Filename: "{app}\evkey.exe"; Parameters: "--settings"; IconFilename: "{app}\evkey.exe"
Name: "{group}\{cm:UninstallProgram,{#AppName}}"; Filename: "{uninstallexe}"

[Run]
; The per-user steps below run as the person who started the installer, so they write their own HKCU.
Filename: "{app}\evkey.exe"; Parameters: "--autostart on"; StatusMsg: "{cm:TaskAutostart}"; Tasks: autostart; Flags: runasoriginaluser runhidden waituntilterminated
Filename: "{app}\evkey.exe"; Parameters: "--hidden"; Description: "{cm:LaunchEVKey}"; Flags: runasoriginaluser nowait postinstall skipifsilent
; Self-update (EVKey passes /relaunch=1 with /SILENT): the wizard page is skipped, so start the new version here.
Filename: "{app}\evkey.exe"; Parameters: "--hidden"; Check: RelaunchRequested; Flags: runasoriginaluser nowait

[UninstallRun]
; runasoriginaluser is not available for uninstall entries: HKCU cleanup runs in the uninstaller's user context.
Filename: "{sysnative}\taskkill.exe"; Parameters: "/F /IM evkey.exe"; Flags: runhidden waituntilterminated; RunOnceId: "KillEVKey"
Filename: "{app}\evkey.exe"; Parameters: "--autostart off"; Flags: runhidden waituntilterminated; RunOnceId: "AutostartOff"

[Code]
function RelaunchRequested: Boolean;
begin
  Result := ExpandConstant('{param:relaunch|0}') = '1';
end;

const
  LegacyTsfKey = 'SOFTWARE\Classes\CLSID\{#LegacyTsfClsid}\InprocServer32';

// Deletes every file in each EVKey version folder except KeepDir (empty = keep none), then the folder.
// Locked files (an old TSF DLL loaded by running apps) are deleted on the next reboot.
procedure RemoveOtherVersions(const KeepDir: String);
var
  FindRec, FindFile: TFindRec;
  Dir, FilePath: String;
begin
  if FindFirst(ExpandConstant('{autopf}\EVKey\*'), FindRec) then
  try
    repeat
      if ((FindRec.Attributes and FILE_ATTRIBUTE_DIRECTORY) <> 0) and (FindRec.Name <> '.') and (FindRec.Name <> '..') then
      begin
        Dir := ExpandConstant('{autopf}\EVKey\') + FindRec.Name;
        if CompareText(Dir, KeepDir) <> 0 then
        begin
          if FindFirst(Dir + '\*', FindFile) then
          try
            repeat
              if (FindFile.Attributes and FILE_ATTRIBUTE_DIRECTORY) = 0 then
              begin
                FilePath := Dir + '\' + FindFile.Name;
                if not DeleteFile(FilePath) then
                  DelayDeleteFile(FilePath, 1);
              end;
            until not FindNext(FindFile);
          finally
            FindClose(FindFile);
          end;
          RemoveDir(Dir);
        end;
      end;
    until not FindNext(FindRec);
  finally
    FindClose(FindRec);
  end;
end;

procedure CurStepChanged(CurStep: TSetupStep);
var
  RC: Integer;
  LegacyDll: String;
begin
  if CurStep = ssInstall then
  begin
    // Stop the running EVKey (old version) so its files are not held. Exit code ignored: may not be running.
    Exec(ExpandConstant('{sysnative}\taskkill.exe'), '/F /IM evkey.exe', '', SW_HIDE, ewWaitUntilTerminated, RC);
    // Versions up to 2.2.x registered a TSF text service; unregistering it also drops the Windows keyboard entry.
    if RegQueryStringValue(HKEY_LOCAL_MACHINE, LegacyTsfKey, '', LegacyDll) and FileExists(LegacyDll) then
      Exec(ExpandConstant('{sysnative}\regsvr32.exe'), '/s /u "' + LegacyDll + '"', '', SW_HIDE, ewWaitUntilTerminated, RC);
    RemoveOtherVersions(ExpandConstant('{app}'));
  end;
end;

procedure CurUninstallStepChanged(CurUninstallStep: TUninstallStep);
begin
  if CurUninstallStep = usPostUninstall then
  begin
    RemoveOtherVersions('');
    RemoveDir(ExpandConstant('{autopf}\EVKey'));
    if not UninstallSilent then
      MsgBox(CustomMessage('UninstallKept'), mbInformation, MB_OK);
  end;
end;
