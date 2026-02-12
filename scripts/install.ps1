$PROGRAM_NAME = "racky"
$REPOSITORY = "DervexDev/racky"

$originalPath = Get-Location

Set-Location "$env:temp"

Function Get-ReleaseInfo {
	param (
		[string]$ApiUrl
	)

	$headers = @{
		'X-GitHub-Api-Version' = '2022-11-28'
	}

	if ($env:GITHUB_TOKEN) {
		$headers['Authorization'] = "token $env:GITHUB_TOKEN"
	}

	try {
		$response = Invoke-RestMethod -Uri $ApiUrl -Headers $headers -ErrorAction Stop
		return $response
	}
	catch {
		throw "Failed to fetch release info: $_"
	}
}

try {
	Write-Host "[1/3] Looking for latest $PROGRAM_NAME release"

	$apiUrl = "https://api.github.com/repos/$REPOSITORY/releases/latest"
	$releaseInfo = Get-ReleaseInfo -ApiUrl $apiUrl

	$versionTag = $releaseInfo.tag_name
	$numericVersion = $versionTag -replace '^v', ''

	$downloadUrl = "https://github.com/$REPOSITORY/releases/download/$versionTag/$PROGRAM_NAME-$numericVersion-windows-x86_64.zip"

	Write-Host "[2/3] Downloading '$PROGRAM_NAME-$numericVersion-windows-x86_64.zip'"

	try {
		Invoke-WebRequest -Uri $downloadUrl -Headers $headers -OutFile racky.zip -ErrorAction Stop
	}
	catch {
		throw "Failed to download from $downloadUrl`: $_"
	}

	try {
		Expand-Archive -Path racky.zip -Force -ErrorAction Stop
	}
	catch {
		throw "Failed to extract racky.zip: $_"
	}

	try {
		if (Test-Path ".\racky\racky.exe") {
			Write-Host "[3/3] Running $PROGRAM_NAME installer"
			Start-Process -FilePath ".\racky\racky.exe" -ArgumentList "install" -Wait -NoNewWindow
		}
		else {
			throw "racky.exe not found in the extracted directory"
		}
	}
	catch {
		throw "Failed to run self-install: $_"
	}

	try {
		Remove-Item racky.zip -ErrorAction SilentlyContinue
		Remove-Item -Recurse -Path .\racky -ErrorAction SilentlyContinue
	}
	catch {
		Write-Warning "Cleanup failed: $_"
	}
}
catch {
	Write-Error "Installation failed: $_"
	exit 1
}
finally {
	Set-Location -Path $originalPath
}
