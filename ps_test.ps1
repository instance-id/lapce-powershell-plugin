#!/usr/bin/env -S pwsh-preview -noProfile -nologo

[Diagnostics.CodeAnalysis.SuppressMessageAttribute('PSAvoidUsingCmdletAliases', '')]
[Diagnostics.CodeAnalysis.SuppressMessageAttribute('PSUseDeclaredVarsMoreThanAssignments', '')]


$CalcCode = @'
public class Calc{
    public int Add(int x,int y){
        return x+y;
    }
    public int Sub(int x,int y){
        return x-y;
    }
    public int Mul(int x,int y){
        return x*y;
    }
    // this method can be accessed without creating the object
    public static float Div(int x,int y){
        return x/y;
    }
}
'@

Add-Type -TypeDefinition $CalcCode -Language CSharp

# calling static method in Powershell
[Calc]::Div(10, 2)
[Calc]::Add(1, 2) # throws an exception

# calling instance method
# instantiate the class to access the non-static methods
$c = New-Object Calc
$c.Add(1, 2)
$c.Mul(3, 3)
$c.Div(16, 4) # throws error


function Find-Files {
    [CmdletBinding()]
    [Diagnostics.CodeAnalysis.SuppressMessageAttribute('PSAvoidUsingCmdletAliases', '')]
    [Diagnostics.CodeAnalysis.SuppressMessageAttribute('PSUseDeclaredVarsMoreThanAssignments', '')]
    param(
        [Parameter()] [string] $glob,
        [Parameter()] [string] $path = (Get-Location))

    if (-not ( $Path -match '^\w:' ) ) {
        $Path = (Get-Location).Path + '\' + $Path
    }

    Write-Host "searching for $Glob in $Path. skip directories."
    # [System.IO.Directory]::EnumerateFiles($Path, $Glob, [System.IO.SearchOption]::AllDirectories)

    foreach ($file in [System.IO.Directory]::EnumerateFiles($path, $glob, [System.IO.SearchOption]::AllDirectories)) {

    }

    [System.IO.EnumerationOptions]$enumOption = [System.IO.EnumerationOptions]::new()
    $enumOption.IgnoreInaccessible = $true
    $enumOption.MatchCasing = [System.IO.MatchCasing]::CaseInsensitive
    $enumOption.MatchType = [System.IO.MatchType]::Simple
    $enumOption.RecurseSubdirectories = -1
    $enumOption.RecurseSubdirectories = $true
    $enumOption.ReturnSpecialDirectories = $false

    <# [System.IO.Enumeration.FileSystemEnumerable] #>
    $files = [System.IO.Directory]::EnumerateFiles($path, $glob, $enumOption)

    $index = 0
    $files | % {
        if ($index -lt 10) {

            $lastWrite = [DateTime]$(get-item $_ ).LastWriteTime
            write-host $lastWrite
        }
        $index++
    }

}

Find-Files '*.ahk' 'C:\Program Files'