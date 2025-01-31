# Docker Installation (Recommended)

The most straightforward method to install the controller is by utilizing a Docker image. Follow the steps below to set it up using Docker Compose:

## Step 1: Create the `docker-compose.yml` File
First, use a text editor to create the `docker-compose.yml` file:
```bash
nano docker-compose.yml
```
Next, add the following content to the file:
```yaml
services:
  controller:
    image: ghcr.io/httprafa/atomic-cloud:latest
    ports:
      - "12892:12892"
    environment:
      - PTERODACTYL=true # Enable Pterodactyl driver installation
      - LOCAL=true       # Enable Local driver installation
    volumes:
      - ./logs:/app/logs
      - ./auth:/app/auth
      - ./configs:/app/configs
      - ./cloudlets:/app/cloudlets
      - ./deployments:/app/deployments
      - ./drivers:/app/drivers
```

## Step 2: Start the Container
To start the container, execute the following command:
```bash
docker compose up
```
