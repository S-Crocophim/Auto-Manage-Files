[Setup]
; AppId uniquely identifies this application.
AppId={{5C0A74B8-9F81-4C52-A932-B8E85E2263F9}
AppName=Auto File Organizer
AppVersion=1.0.1
AppPublisher=Px0
AppPublisherURL=https://github.com/S-Crocophim/Auto-Manage-Files
DefaultDirName={autopf}\Auto File Organizer
DisableProgramGroupPage=yes
PrivilegesRequired=lowest
OutputDir=dist
OutputBaseFilename=AutoFileOrganizer_Installer
Compression=lzma
SolidCompression=yes
WizardStyle=modern

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked

[Files]
Source: "dist\AutoFileOrganizer_Full\AutoFileOrganizer_Full.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "dist\AutoFileOrganizer_Full\*"; DestDir: "{app}"; Flags: ignoreversion recursesubdirs createallsubdirs

[Icons]
Name: "{autoprograms}\Auto File Organizer"; Filename: "{app}\AutoFileOrganizer_Full.exe"
Name: "{autodesktop}\Auto File Organizer"; Filename: "{app}\AutoFileOrganizer_Full.exe"; Tasks: desktopicon

[Run]
Filename: "{app}\AutoFileOrganizer_Full.exe"; Description: "{cm:LaunchProgram,Auto File Organizer}"; Flags: nowait postinstall skipifsilent
