# Simple Budget [![Github Actions](https://action-badges.now.sh/corybuecker/simple-budget)](https://github.com/corybuecker/simple-budget/actions) [![Coverage Status](https://coveralls.io/repos/github/corybuecker/simple-budget/badge.svg?branch=master)](https://coveralls.io/github/corybuecker/simple-budget?branch=master) ![GitHub](https://img.shields.io/github/license/corybuecker/simple-budget.svg)

Simple budget is a very simple, and opinionated, budgeting tool. I use it for my personal finances. It is based on the concept of envelope saving, but with more automation. I practice a "daily" budget, where money for long-term goals is amortized and removed from my remaining funds on a daily basis.

A web-based demo will be added soon. In the meantime, you can run the application locally with Docker.

    docker-compose build
    docker-compose run web mix ecto.setup
    docker-compose up

Once the containers have started, just browse to localhost:4000 in your browser. Authentication is disabled in the local development environment.

### TODOs

- [X] Make authentication more user-friendly
- [ ] Document how to set up locally with Google and/or seeds
- [ ] Setup demo site
- [ ] Associate accounts, goals, etc. to user records
- [ ] Add instructions for running locally without Docker
- [ ] Add user guide and definitions
- [ ] Make editor UI more user-friendly
- [ ] Import transactions with Spectre or Plaid
- [ ] Add back authentication in the local environment, probably replacing the token fetcher with another implementation
- [ ] Add visualizations for daily reporting
- [ ] Move elixir tests to doc-based and document modules
- [ ] Refactor Elm code to conventions, e.g. pipes, imports, etc.
- [ ] Add sorting to tables
