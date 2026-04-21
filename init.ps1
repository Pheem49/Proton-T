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

function Global:Set-Location {
    [CmdletBinding(DefaultParameterSetName='Path', SupportsTransactions=$true)]
    param(
        [Parameter(ParameterSetName='Path', Position=0, ValueFromPipelineByPropertyName=$true)]
        [string]$Path,

        [Parameter(ParameterSetName='LiteralPath', Mandatory=$true, ValueFromPipelineByPropertyName=$true)]
        [string]$LiteralPath,

        [switch]$PassThru,
        [string]$StackName
    )

    if ($PSCmdlet.ParameterSetName -eq 'LiteralPath') {
        Microsoft.PowerShell.Management\Set-Location -LiteralPath $LiteralPath -PassThru:$PassThru -StackName $StackName
    } else {
        Microsoft.PowerShell.Management\Set-Location -Path $Path -PassThru:$PassThru -StackName $StackName
    }

    try {
        proton-t add "$PWD" | Out-Null
    } catch {}
}

try { proton-t add "$PWD" | Out-Null } catch {}

# Completion
Register-ArgumentCompleter -CommandName t -ParameterName Keywords -ScriptBlock {
    param($commandName, $parameterName, $wordToComplete, $commandAst, $fakeBoundParameters)
    $keywords = @($commandAst.CommandElements | Select-Object -Skip 1 | ForEach-Object { $_.Extent.Text })
    if ($keywords.Count -eq 0 -and $wordToComplete) {
        $keywords = @($wordToComplete)
    }

    $completions = if ($keywords.Count -gt 0) {
        proton-t complete $keywords
    } else {
        proton-t complete
    }

    $completions | ForEach-Object {
        [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $_)
    }
}
