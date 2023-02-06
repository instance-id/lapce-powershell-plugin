### Lapce - PowerShell Language Server Plugin

In development, very basics are working (syntax highlighting, hover details of types, etc), but there are some bugs.

! Currently Only Tested On Linux !

Immediately noticed:

- Pwsh process does not exit when the extension is deactivated/Lapce is closed, so you have to kill it manually.
- The current line number sent to the LSP is sometimes out of range, causing syntax highlighting to be off.

<img src="media/pwsh_lapce.png" width="800px">

---
 
### Instructions:

Download the following:
https://github.com/PowerShell/PowerShellEditorServices/releases/latest/download/PowerShellEditorServices.zip

Extract the folder to the following location:

Linux/Mac:

- `~/.local/share/lapce-stable/plugins/instanceid.lapce-powershell/`

Windows:

- `%appdata%\Local\lapce\Lapce-Stable\data\plugins\instanceid.lapce-powershell\`
