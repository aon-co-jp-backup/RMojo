; RMojo installer for Windows (Inno Setup 6). Native rmojo.exe, no WSL needed.
; Build: ISCC.exe /DAppVer=0.3.0 /DBinDir=..\..\target\release installers\windows\rmojo.iss  -> Output\RMojo-Setup.exe
#ifndef AppVer
#define AppVer "0.3.0"
#endif
#ifndef BinDir
#define BinDir "..\..\target\release"
#endif
[Setup]
AppId={{3B7C2E4A-6F10-4B59-8A32-0C9D5E1F7A64}
AppName=RMojo (Mojo-style language, Python syntax, Rust inside)
AppVersion={#AppVer}
AppPublisher=aon-co-jp
AppPublisherURL=https://github.com/aon-co-jp/RMojo
DefaultDirName={localappdata}\Programs\RMojo
DisableProgramGroupPage=yes
PrivilegesRequired=lowest
OutputBaseFilename=RMojo-Setup
OutputDir=Output
Compression=lzma2
SolidCompression=yes
WizardStyle=modern
LicenseFile=..\..\LICENSE
ChangesEnvironment=yes
UninstallDisplayName=RMojo
[Files]
Source: "{#BinDir}\rmojo.exe"; DestDir: "{app}\bin"; Flags: ignoreversion
Source: "..\..\examples\*"; DestDir: "{app}\examples"; Flags: ignoreversion recursesubdirs
Source: "..\..\README.md"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\..\LICENSE"; DestDir: "{app}"; Flags: ignoreversion
[Code]
const EnvKey = 'Environment';
procedure AddToPath(Dir: string);
var P: string;
begin
  if not RegQueryStringValue(HKCU, EnvKey, 'Path', P) then P := '';
  if Pos(';' + Lowercase(Dir) + ';', ';' + Lowercase(P) + ';') = 0 then
  begin
    if (P <> '') and (P[Length(P)] <> ';') then P := P + ';';
    RegWriteExpandStringValue(HKCU, EnvKey, 'Path', P + Dir);
  end;
end;
procedure RemoveFromPath(Dir: string);
var P, Q: string; I: Integer;
begin
  if not RegQueryStringValue(HKCU, EnvKey, 'Path', P) then Exit;
  Q := ';' + P + ';';
  I := Pos(';' + Lowercase(Dir) + ';', Lowercase(Q));
  if I > 0 then
  begin
    Delete(Q, I, Length(Dir) + 1);
    if (Length(Q) > 0) and (Q[1] = ';') then Delete(Q, 1, 1);
    if (Length(Q) > 0) and (Q[Length(Q)] = ';') then Delete(Q, Length(Q), 1);
    RegWriteExpandStringValue(HKCU, EnvKey, 'Path', Q);
  end;
end;
procedure CurStepChanged(CurStep: TSetupStep);
begin
  if CurStep = ssPostInstall then AddToPath(ExpandConstant('{app}\bin'));
end;
procedure CurUninstallStepChanged(CurUninstallStep: TUninstallStep);
begin
  if CurUninstallStep = usPostUninstall then RemoveFromPath(ExpandConstant('{app}\bin'));
end;
