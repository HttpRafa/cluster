#!/usr/bin/env bash

# Check if the server jar already exists
if [ -f "${SERVER_JARFILE}" ]; then
    echo "The ${SERVER_JARFILE} already exists. Skipping script execution."
    exit 0
fi

if [ -n "${DL_PATH}" ]; then
	echo -e "Using supplied download url: ${DL_PATH}"
	DOWNLOAD_URL=$(eval echo $(echo ${DL_PATH} | sed -e 's/{{/${/g' -e 's/}}/}/g'))
else
	# Fetch the versions and check for the given version
	VERSIONS=$(curl -s https://api.papermc.io/v2/projects/${PROJECT} | grep -oP '(?<="versions":\[)[^]]*' | tr ',' '\n' | tr -d '"')
	VER_EXISTS=$(echo "${VERSIONS}" | grep -x "${VERSION}" || echo "")

	LATEST_VERSION=$(echo "${VERSIONS}" | tail -n 1)

	if [ -n "${VER_EXISTS}" ]; then
		echo -e "Version is valid. Using version ${VERSION}"
	else
		echo -e "Specified version not found. Defaulting to the latest ${PROJECT} version"
		VERSION=${LATEST_VERSION}
	fi

	# Fetch the builds and check for the given build
	BUILDS=$(curl -s https://api.papermc.io/v2/projects/${PROJECT}/versions/${VERSION} | grep -oP '(?<="builds":\[)[^]]*' | tr ',' '\n')
	BUILD_EXISTS=$(echo "${BUILDS}" | grep -x "${BUILD_NUMBER}" || echo "")

	LATEST_BUILD=$(echo "${BUILDS}" | tail -n 1)

	if [ -n "${BUILD_EXISTS}" ]; then
		echo -e "Build is valid for version ${VERSION}. Using build ${BUILD_NUMBER}"
	else
		echo -e "Using the latest ${PROJECT} build for version ${VERSION}"
		BUILD_NUMBER=${LATEST_BUILD}
	fi

	JAR_NAME=${PROJECT}-${VERSION}-${BUILD_NUMBER}.jar

	echo "Version being downloaded"
	echo -e "MC Version: ${VERSION}"
	echo -e "Build: ${BUILD_NUMBER}"
	echo -e "JAR Name of Build: ${JAR_NAME}"
	DOWNLOAD_URL=https://api.papermc.io/v2/projects/${PROJECT}/versions/${VERSION}/builds/${BUILD_NUMBER}/downloads/${JAR_NAME}
fi

echo "Running curl -o ${SERVER_JARFILE} ${DOWNLOAD_URL}"

curl -o ${SERVER_JARFILE} ${DOWNLOAD_URL}

# Only execute if PROJECT is "paper"
if [ "${PROJECT}" == "paper" ]; then
	echo "Installing required client plugin..."
	curl -o cloud.jar -L https://github.com/HttpRafa/atomic-cloud/releases/latest/download/paper-client.jar
	mkdir -p plugins/
	mv cloud.jar plugins/
	echo "Installed required plugin"
	echo "Preparing server..."
	echo "eula=true" >> eula.txt
	echo "accepts-transfers=true" >> server.properties
	echo "settings:" >> bukkit.yml
	echo "  connection-throttle: -1" >> bukkit.yml
	echo "Ready to start!"
	exit 0
fi