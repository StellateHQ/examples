# gcdn-push-demo

This is a demo project, that regularly pushes a GraphQL schema to a GraphCDN service via various CI provider.

The service is configured to use `https://countries.trevorblades.com/` as backend service, the schema is taken from [trevorblades/countries](https://github.com/trevorblades/countries).

## Create a Personal Access Token

The `stellate` CLI uses _Personal Access Tokens_ to authenticate when used in script mode. By default it expects that token in an environment variable called `STELLATE_TOKEN`.

To create that token, head over to [your dashboard](https://stellate.co/app/settings), provide a descriptive name for the new token and click the _Generate new token_ button. Make sure to copy the token value before you continue, as it will not be displayed again.

## GitHub Actions

[![GitHub Actions Status](https://github.com/StellateHQ/gcdn-push-demo/actions/workflows/stellate.yml/badge.svg)](https://github.com/StellateHQ/gcdn-push-demo/actions/workflows/stellate.yml)

Configured in [`.github/workflows/stellate.yml`](.github/workflows/stellate.yml)

Configure your GraphCDN token as an _Action Secret_. You can read more about how to do this on GitHubs documentation page on [Encrypted Secrets](https://docs.github.com/en/actions/security-guides/encrypted-secrets).

```yaml
name: Stellate
on: 
  push: 
    branches: 
      - main 

jobs: 
  main: 
    runs-on: ubuntu-latest
    steps: 
      - uses: actions/checkout@v3
      - uses: actions/setup-node@v3
        with: 
          node-version: 'lts/*'
          check-latest: true 
          cache: 'npm'
      - name: Push to Stellate
        run: npx stellate push 
        env: 
          STELLATE_TOKEN: ${{ secrets.STELLATE_TOKEN }}
```

## GitLab CI

![GitLab CI Status](https://gitlab.com/mlocher/gcdn-push-demo/badges/main/pipeline.svg)


Configured in [`.gitlab-ci.yml`](.gitlab-ci.yml)

Configure your Stellate token as a masked and protected CI/CD variable. You'll find that section in your project settings, within the CI/CD settings page. See GitLabs documentation on [Project Variables](https://docs.gitlab.com/ee/ci/variables/#add-a-cicd-variable-to-a-project) for more information.

If your GitLab subscription supports `secrets`, you can also make use of an external secrets store to provide the Stellate token. Please see the [secrets documentation](https://docs.gitlab.com/ee/ci/yaml/#secrets) for more information on how to implement this.

```yml
config_push:
  image: node:lts
  cache:
    paths:
      - node_modules
  rules:
    - if: '$CI_COMMIT_BRANCH =~ /^main/'
  script:
    - npx stellate push 
```

## CircleCI

[![CircleCI Status](https://dl.circleci.com/status-badge/img/gh/StellateHQ/gcdn-push-demo/tree/main.svg?style=svg&circle-token=2734aba6c4f384afcfd769e99229f0e1c127e94c)](https://dl.circleci.com/status-badge/redirect/gh/StellateHQ/gcdn-push-demo/tree/main)

Configured in [`.circleci/config.yml`](.circleci/config.yml)

Configure your Stellate token as an environment variable in a CircleCI [Context](https://circleci.com/docs/2.0/contexts/) and make sure that the jobs are configured to use that context.

```yaml
version: 2.1

jobs: 
  stellate: 
    docker: 
      - image: cimg/node:lts
    steps: 
      - checkout 
      - run: npx stellate push 

workflows:
  version: 2.1
  config_push:
    jobs:
      - stellate:
          context:
            - stellate
```