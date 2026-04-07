function t {
    param([Parameter(ValueFromRemainingArguments=$true)]$Keywords)
    if ($Keywords.Count -eq 0) {
        proton-t list
        return
    }
    if ($Keywords[0] -eq "-") {
        Pop-Location
        return
    }
    if ($Keywords[0] -eq "..") {
        Set-Location ..
        return
    }

    $result = proton-t query $Keywords
    if ($result) {
        if ($env:_PT_ECHO -eq "1") { Write-Host $result }
        Set-Location $result
    } else {
        Write-Host "proton-t: no match found for '$Keywords'" -ForegroundColor Red
    }
}

function ti {
    param([Parameter(ValueFromRemainingArguments=$true)]$Keywords)
    $result = proton-t interactive $Keywords
    if ($result) {
        if ($env:_PT_ECHO -eq "1") { Write-Host $result }
        Set-Location $result
    }
}

# Hook to track directory changes
if (-not (Get-Variable -Name "old_prompt" -ErrorAction SilentlyContinue)) {
    $Global:old_prompt = $ExecutionContext.InvokeCommand.GetCommand('prompt', 'Function')
}

function Global:prompt {
    # Run the tracker silently
    try {
        proton-t add "$PWD"
    } catch {}
    
    if ($Global:old_prompt) {
        & $Global:old_prompt
    } else {
        "PS $($executionContext.SessionState.Path.CurrentLocation)> "
    }
}
