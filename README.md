# Overseerr/Jellyseerr Media Cleaner

This Rust application scans media items from **Overseerr** and **Jellyseerr**, checks whether the corresponding media entries still exist in **Sonarr** or **Radarr**, and deletes entries from Overseerr/Jellyseerr if the media no longer exists in the associated service.

The project is containerized with Docker and configured via environment variables.

---

## Features

✅ Connects to:
- Overseerr  
- Jellyseerr  
- Sonarr  
- Radarr  

✅ Checks for missing media items (TV/Movies) in Sonarr/Radarr

✅ Deletes stale media entries from Overseerr/Jellyseerr if the corresponding media does not exist

---

## Environment Variables

Set the following environment variables in your Docker environment (via `docker run -e` flags, or via `docker-compose` `environment` entries):

| Variable            | Description                                                        |
|----------------------|--------------------------------------------------------------------|
| `OVERSEERR_URL`      | Base URL of your Overseerr instance (e.g. `http://overseerr:5055/api/v1`) |
| `OVERSEERR_KEY`      | API key for your Overseerr instance                                |
| `JELLYSEERR_URL`     | Base URL of your Jellyseerr instance (e.g. `http://jellyseerr:5055/api/v1`) |
| `JELLYSEERR_KEY`     | API key for your Jellyseerr instance                               |
| `SONARR_URL`         | Base URL of your Sonarr instance (e.g. `http://sonarr:8989/api/v3`) |
| `SONARR_KEY`         | API key for your Sonarr instance                                   |
| `RADARR_URL`         | Base URL of your Radarr instance (e.g. `http://radarr:7878/api/v3`) |
| `RADARR_KEY`         | API key for your Radarr instance                                   |

---

## Usage

### Docker Run Example

```bash
docker run \
  -e OVERSEERR_URL="http://overseerr:5055/api/v1" \
  -e OVERSEERR_KEY="your_overseerr_api_key" \
  -e JELLYSEERR_URL="http://jellyseerr:5055/api/v1" \
  -e JELLYSEERR_KEY="your_jellyseerr_api_key" \
  -e SONARR_URL="http://sonarr:8989/api/v3" \
  -e SONARR_KEY="your_sonarr_api_key" \
  -e RADARR_URL="http://radarr:7878/api/v3" \
  -e RADARR_KEY="your_radarr_api_key" \
  your_docker_image_name
```

### docker-compose Example

Here’s an example `docker-compose.yml` snippet:

```yaml
services:
  cleanseerr:
    image: tefox/cleanseerr:latest
    environment:
      OVERSEERR_URL: http://overseerr:5055/api/v1
      OVERSEERR_KEY: your_overseerr_api_key
      JELLYSEERR_URL: http://jellyseerr:5055/api/v1
      JELLYSEERR_KEY: your_jellyseerr_api_key
      SONARR_URL: http://sonarr:8989/api/v3
      SONARR_KEY: your_sonarr_api_key
      RADARR_URL: http://radarr:7878/api/v3
      RADARR_KEY: your_radarr_api_key
```

---

## How It Works

1. **Retrieve Media Items**  
   The app fetches a paginated list of media in a `processing` state from Overseerr and Jellyseerr.

2. **Check Existence in Sonarr/Radarr**  
   For each media item:
   - If it’s a TV show → check Sonarr for the corresponding series ID
   - If it’s a Movie → check Radarr for the corresponding movie ID

3. **Delete Stale Entries**  
   If Sonarr/Radarr respond with an error (media missing), the app deletes the media item from Overseerr/Jellyseerr via a DELETE request.

---

## Notes

- The app only processes media items in the `processing` filter.
- Pagination is handled automatically.
- If no media is deleted in the current page, it proceeds to the next page until all pages are scanned.

---

## Build Locally

If you’d like to run the app outside Docker:

```bash
cargo build --release
```

Then run with environment variables set:

```bash
OVERSEERR_URL=http://overseerr:5055/api/v1 \
OVERSEERR_KEY=your_overseerr_api_key \
JELLYSEERR_URL=http://jellyseerr:5055/api/v1 \
JELLYSEERR_KEY=your_jellyseerr_api_key \
SONARR_URL=http://sonarr:8989/api/v3 \
SONARR_KEY=your_sonarr_api_key \
RADARR_URL=http://radarr:7878/api/v3 \
RADARR_KEY=your_radarr_api_key \
./target/release/your_binary_name
```

---

## License

[MIT](LICENSE)
