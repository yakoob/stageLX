# GDTF Fixture Corpus

This directory holds real `.gdtf` fixture files for integration testing.

## Running the corpus test

```bash
cargo test -p stagelx-gdtf --test corpus
```

If the directory is empty, the test prints a message and passes.

## Populating the corpus

### Option A: Manual download from GDTF Share

1. Visit <https://gdtf-share.com>
2. Search for fixtures by manufacturer
3. Download `.gdtf` files
4. Copy them into this directory

### Option B: API download (requires account)

The GDTF Share provides a REST API:

```bash
# 1. Login (requires valid credentials)
curl -c session.txt -X POST https://gdtf-share.com/apis/public/login.php \
  -d "email=YOUR_EMAIL" -d "password=YOUR_PASSWORD"

# 2. Get fixture list
curl -b session.txt https://gdtf-share.com/apis/public/getList.php > fixtures.json

# 3. Download a specific fixture by revision ID
curl -b session.txt \
  "https://gdtf-share.com/apis/public/downloadFile.php?rid=12345" \
  -o fixture.gdtf
```

See <https://www.gdtf.eu/gdtf/share_api/share-api/> for full API documentation.

## Recommended cross-manufacturer sample

For good coverage, aim for a mix of fixture types:
- Moving heads (pan/tilt, multiple DMX modes)
- LED washes (RGB/RGBW mixing)
- Spot fixtures (gobo wheels, prisms)
- Static pars (dimmer only)
- Laser fixtures (non-standard attributes)

## Notes

- `.gdtf` files are **gitignored** — do not commit them to the repository.
- Files are typically 1–10 MB each (ZIP archives containing XML + 3D models + gobo images).
