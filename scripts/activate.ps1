# Sets the current user's default input override to EVKey (en-US profile), or clears it with -Off.
# Used by smoke tests. Restores nothing by itself: run with -Off to go back.
param([switch]$Off)
$ErrorActionPreference = 'Stop'
$tip = '0409:{B5F8D54F-6BAD-46D4-8DBB-FFE03BF4D2F4}{91566F64-0E09-4724-93DA-6C798B765C4E}'
if ($Off) {
    Set-WinDefaultInputMethodOverride -InputTip '0409:00000409'
    Write-Host 'Override set to 0409:00000409 (en-US keyboard).'
} else {
    Set-WinDefaultInputMethodOverride -InputTip $tip
    Write-Host "Override set to $tip"
}
Get-WinDefaultInputMethodOverride
