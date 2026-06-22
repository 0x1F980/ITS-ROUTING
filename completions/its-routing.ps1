# PowerShell completion for its-routing — UES Monocell Pool CLI
# Usage: . ./completions/its-routing.ps1

using namespace System.Management.Automation
using namespace System.Management.Automation.Language

Register-ArgumentCompleter -Native -CommandName 'its-routing' -ScriptBlock {
    param($wordToComplete, $commandAst, $cursorPosition)

    $tokens = @(
        foreach ($element in $commandAst.CommandElements) {
            if ($element -is [StringConstantExpressionAst] -and
                $element.StringConstantType -eq [StringConstantType]::BareWord) {
                $element.Value
            }
        }
    )

    $command = ($tokens -join ';')

    $completions = @(switch -Regex ($command) {
        '^its-routing$' {
            @(
                @('client-send', 'Publish wire to UES Monocell Pool'),
                @('client-receive', 'Harvest pool and reconstruct'),
                @('time-lock', 'Generate time-lock puzzle'),
                @('time-unlock', 'Solve time-lock puzzle'),
                @('time-deny', 'Decoy time-lock unlock'),
                @('fingerprint-erasure', 'Offline provenance erasure'),
                @('client-export-share', 'Export SSS share'),
                @('client-import-share', 'Import SSS share'),
                @('start-node', 'Dev-only onion daemon')
            ) | ForEach-Object {
                [CompletionResult]::new($_[0], $_[0], [CompletionResultType]::ParameterValue, $_[1])
            }
            '-c', '--config', '-h', '--help', '-v', '--version' | ForEach-Object {
                [CompletionResult]::new($_, $_, [CompletionResultType]::ParameterName, $_)
            }
            break
        }
        '^its-routing;start-node' {
            '-p', '--port', '-r', '--chaff-rate' | ForEach-Object {
                [CompletionResult]::new($_, $_, [CompletionResultType]::ParameterName, $_)
            }
            break
        }
        '^its-routing;client-send' {
            '-m', '--msg', '-f', '--file', '-d', '--dest', '--pool', '--no-pool', '--aeh', '--continuous', '--ratchet-seed-file', '--fingerprint-erasure', '--mailbox-fingerprint' | ForEach-Object {
                [CompletionResult]::new($_, $_, [CompletionResultType]::ParameterName, $_)
            }
            break
        }
        '^its-routing;client-receive' {
            '--pool', '--no-pool', '--aeh', '--continuous', '--ratchet-seed-file', '-o', '--out', '--timeout-secs', '--mailbox-fingerprint', '--mailbox-strict' | ForEach-Object {
                [CompletionResult]::new($_, $_, [CompletionResultType]::ParameterName, $_)
            }
            break
        }
        '^its-routing;time-lock' {
            '-f', '--file', '-e', '--epochs', '-o', '--out' | ForEach-Object {
                [CompletionResult]::new($_, $_, [CompletionResultType]::ParameterName, $_)
            }
            break
        }
        '^its-routing;time-unlock' {
            '-p', '--puzzle', '-o', '--out' | ForEach-Object {
                [CompletionResult]::new($_, $_, [CompletionResultType]::ParameterName, $_)
            }
            break
        }
        '^its-routing;time-deny' {
            '-p', '--puzzle', '-d', '--decoy', '-o', '--out' | ForEach-Object {
                [CompletionResult]::new($_, $_, [CompletionResultType]::ParameterName, $_)
            }
            break
        }
    })

    $completions | Where-Object { $_.CompletionText -like "$wordToComplete*" }
}
