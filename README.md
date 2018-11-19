# Simple Budget [![CircleCI](https://circleci.com/gh/corybuecker/simple-budget/tree/master.svg?style=svg)](https://circleci.com/gh/corybuecker/simple-budget/tree/master) [![Coverage Status](https://coveralls.io/repos/github/corybuecker/simple-budget/badge.svg?branch=master)](https://coveralls.io/github/corybuecker/simple-budget?branch=master)

Simple budget is a very simple, and opinionated, budgeting tool. I use it for my personal finances. It is based on the concept of envelope saving, but with more automation. I practice a "daily" budget, where money for long-term goals is amortized and removed from my remaining funds on a daily basis.

A web-based demo will be added soon. In the meantime, you can run the application locally with Docker.

    docker-compose build
    docker-compose run web mix ecto.setup
    docker-compose up

### TODOs

- [ ] Make authentication more user-friendly
- [ ] Setup demo site
- [ ] Add instructions for running locally without Docker
- [ ] Add user guide and definitions
- [ ] Make editor UI more user-friendly
