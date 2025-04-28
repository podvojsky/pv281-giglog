# PV281 - Team Project - Giglog 🎸

[![pipeline status](https://gitlab.fi.muni.cz/xpodvojs/pv281-giglog/badges/main/pipeline.svg)](https://gitlab.fi.muni.cz/xpodvojs/pv281-giglog/-/commits/main)

## Live demo (FI MUNI VPN is needed)

[http://172.26.8.13/](http://172.26.8.13/)

Admin account credentials

- Username: `admin`
- Password: `iloverust`

All other testing accounts have the same password: `iloverust`.

## Prerequisites

- Rustc >= 1.81.0 (eeb90cda1 2024-09-04)
- Cargo >= 1.81.0 (2dbb1af80 2024-08-20)
- Node >= v20.10.0
- npm >= 10.2.3

## Installation

```
npm install
```

### If you want live reload in development

```
cargo install cargo-watch
```

## Usage

### Development

1. Copy `.env.template` file and rename it as `.env`:

    ```text
    cp .env.template .env
    ```

2. If necessary modify **environment variables** in `.env` file.
   - Leave defaults if you want to develop with local database in docker.

3. Run Postgresql database in docker:

     - Make sure that you don't have any other docker containers that would interfere with database running on port `5432`.

    ```text
    docker compose up
    ```

4. Run SQLx migrations:

    ```text
    sqlx migrate run
    ```

5. Build and run the app, watch for changes (requires cargo-watch):

    ```text
    cargo watch -x run
    ```

6. Generate static styles (*tailwind*), watch for changes:

    ```text
    npm run build:css
    ```

### Production

There are two ways how to make a production build. First one is to generate a docker image, second one is to manually build the app.

#### Docker

1. Build the image:

    ```sh
    docker build --build-arg DATABASE_URL=postgresql://postgres:example@localhost:5432 -t pv281-giglog .
    ```

2. Run the container:

    ```sh
    docker run -p [YOUR_PORT]:3000 pv281-giglog
    ```

3. The application should be running on `http://localhost:[YOUR_PORT]`.

#### Manual

1. Build the application (in *production* mode):

   ```text
   cargo build --release --verbose
   ```

2. Generate static styles:

    ```text
    tailwindcss -i ./assets/styles.css -o ./public/styles.css --minify
    ```

3. Put generated styles into `./public` folder.
   - Final executable and public folder with styles and static assets needs to be in the same folder (`deploy`).

        ```text
        deploy/
        ├─ public/
        │  ├─ styles.css
        ├─ pv281-giglog
        ```

4. Run the executable `./pv281-giglog` or `pv281-giglog.exe` if you are on Windows.

5. The application should be running on `http://localhost:[ENV_PORT]`.

test deploy 2
