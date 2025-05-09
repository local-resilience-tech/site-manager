# LoRes Node

Manage a [Node](https://jade.hopepunk.me/posts/sites-the-main-component-of-merri-bek-tech/)
in a LoRes Mesh.

LoRes Mesh allows you to run a network of low powered, resilient local web hosts that work with the internet, but are not dependent on it.

Developed by [Merri-bek Tech](https://www.merri-bek.tech/), but useful for all similar organisations.

# Development

## Tech Stack

### Backend

The backend is a rust app, using the [Rocket framework](https://rocket.rs/). The rust package management tool "cargo" is used. To fetch what you need and start the server, run `cargo run`.

## Frontend

The frontend (web interface) is built using React, using the [Vite](https://vitejs.dev/) as the tooling to build and run. Packages are managed using npm. It's also heavily dependent on several other library choices:

- [Chakra UI](https://chakra-ui.com/) For the component library and styling

Running the frontend us done with `npm run dev`

## Generating a release

We use [Release it](https://github.com/release-it/release-it). For the first time, you'll need to install it with 'npm install' from the root directory of this project. After that, to run it, use `npm run release`

# Running

## Running Local Version in Development Mode

When developing, it is recommended that you run the backend and frontend seperately.

Front End: Open a terminal in the frontend directory and run `npm run dev`. This will hot-reload any changes.
Back End: Open a terminal in the backend directory and run `cargo run`. If you make changes to the backend, you'll need to halt it with CTRL-C and re-run.

To access it in the browser, access the front end at the port that vite uses by default, which is http://localhost:5173/. The backend is at http://localhost:8000/.

## Running Local Version in Release Mode

To run the app locally in release mode you can build it and run it using docker.

```
sudo docker build -t dev/site-manager:latest . && sudo docker run -p 8000:80 dev/site-manager:latest
```

In release mode, the front end is not rendered, it's just built and placed in the docker container at the dir `/app/frontend`. The backend rust app builds an executable in `/app/backend` which is the command run by docker.

The backend will serve up the frontend, which only happens because the environment variable `ROCKET_FRONTEND_ASSET_PATH` is set in the docker container. The rust app is running on port 80 in the container, which is why, when running it, you may want to map that to a custom port.

## Running Published Version

On every release, a version is published to Docker Hub at:
https://hub.docker.com/r/resilientlocaltech/site-manager

This can be run using a single docker command, like:
`sudo docker run -p 8000:80 resilientlocaltech/site-manager:latest`

However, since this app is likely to gain other dependent services, it is recommended that you use docker swarm.

### Using Docker Compose

This app is designed to be deployed using Docker Swarm. As such it provides a `compose.yml` file based on the [older v3 of the docker compose format](<https://github.com/docker/compose/blob/v1/docs/Compose%20file%20reference%20(legacy)/version-3.md>).

Before trying this with docker swarm, it's worth trying to run it with docker compose. In the root directory of the project, run:

`docker compose up`

The app should then be running at http//localhost:8000.

### Using Docker Swarm

This app is designed to be deployed on Raspberry Pis used in a docker swarm.

To deploy it, fetch the latest compose file using:

`curl https://raw.githubusercontent.com/merri-bek-tech/site-manager/refs/heads/main/compose.yml > site-manager.yml`

The, providing you have a docker swarm running, use:

`docker stack deploy -c ./site-manager.yml site-manager`

The app should then be running on post 8000 of your pi. We recommend using [swarmpit](https://swarmpit.io/) to monitor your swarm.

# Database Handling

The Backend uses an SQLite database. The rust integration uses a library called `sqlx` that handles queries and database migrations, and also performs compile time checking of SQL queries against the DB structure. There are some command-line tools to help out with this

## SQLX Command Line Tools

### Installing

You'll need the following pre-requisites on linux:

`sudo apt install pkg-config libssl-dev`

Then install with `cargo install sqlx-cli`

### Useful commands

Re-create the database:
`DATABASE_URL=sqlite:./main.sqlite cargo sqlx database reset`

Re-build the query indexes:
`DATABASE_URL=sqlite:./main.sqlite cargo sqlx prepare`
