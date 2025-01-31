# Check if the server jar already exists
if (Test-Path $env:SERVER_JARFILE) {
    Write-Host "The $env:SERVER_JARFILE already exists. Skipping script execution."
    exit 0
}

if ($env:DL_PATH) {
    Write-Host "Using supplied download URL: $($env:DL_PATH)"
    $DOWNLOAD_URL = $env:DL_PATH -replace "{{", '$(' -replace "}}", ')'
} else {
    # Fetch the versions and check for the given version
    $VERSIONS = (Invoke-RestMethod -Uri "https://api.papermc.io/v2/projects/$env:PROJECT").versions
    $VER_EXISTS = $VERSIONS -contains $env:VERSION

    $LATEST_VERSION = $VERSIONS[-1]

    if ($VER_EXISTS) {
        Write-Host "Version is valid. Using version $env:VERSION"
    } else {
        Write-Host "Specified version not found. Defaulting to the latest $env:PROJECT version"
        $env:VERSION = $LATEST_VERSION
    }

    # Fetch the builds and check for the given build
    $BUILDS = (Invoke-RestMethod -Uri "https://api.papermc.io/v2/projects/$env:PROJECT/versions/$env:VERSION").builds
    $BUILD_EXISTS = $BUILDS -contains $env:BUILD_NUMBER

    $LATEST_BUILD = $BUILDS[-1]

    if ($BUILD_EXISTS) {
        Write-Host "Build is valid for version $env:VERSION. Using build $env:BUILD_NUMBER"
    } else {
        Write-Host "Using the latest $env:PROJECT build for version $env:VERSION"
        $env:BUILD_NUMBER = $LATEST_BUILD
    }

    $JAR_NAME = "$env:PROJECT-$env:VERSION-$env:BUILD_NUMBER.jar"

    Write-Host "Version being downloaded"
    Write-Host "MC Version: $env:VERSION"
    Write-Host "Build: $env:BUILD_NUMBER"
    Write-Host "JAR Name of Build: $JAR_NAME"
    $DOWNLOAD_URL = "https://api.papermc.io/v2/projects/$env:PROJECT/versions/$env:VERSION/builds/$env:BUILD_NUMBER/downloads/$JAR_NAME"
}

Write-Host "Running Invoke-WebRequest -Uri $DOWNLOAD_URL -OutFile $env:SERVER_JARFILE"

Invoke-WebRequest -Uri $DOWNLOAD_URL -OutFile $env:SERVER_JARFILE

if ($env:PROJECT -eq "paper") {
    Write-Host "Installing required client plugin..."
    Invoke-WebRequest -Uri "https://github.com/HttpRafa/atomic-cloud/releases/latest/download/paper-client.jar" -OutFile "paper-client.jar"
    New-Item -ItemType Directory -Path "plugins" -Force | Out-Null
    Move-Item -Path "paper-client.jar" -Destination "plugins\"
    Write-Host "Installed required plugin"

    Write-Host "Preparing server..."
    Add-Content -Path "eula.txt" -Value "eula=true"
    Add-Content -Path "server.properties" -Value "accepts-transfers=true"
    Add-Content -Path "bukkit.yml" -Value "settings:"
    Add-Content -Path "bukkit.yml" -Value "  connection-throttle: -1"
    Write-Host "Ready to start!"
}