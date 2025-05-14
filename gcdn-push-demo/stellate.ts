import { Config } from 'stellate'

function randomInt(min: number, max: number): number {
  return Math.floor(Math.random() * (max - min + 1)) + min
}

const config: Config = {
  config: {
    schema: './schema.graphql',
    enablePlayground: true,
    rootTypeNames: {
      query: 'Query',
    },
    rules: [
      {
        types: ['Query'],
        maxAge: randomInt(600, 900),
        swr: randomInt(300, 600),
        description: 'Cache everything for a random amount of time',
      },
    ],
    name: 'gcdn-push-demo',
    originUrl: 'https://countries.trevorblades.com/',
    environments: {
      staging: {
        name: 'gcdn-push-demo-staging',
        schema: './schema.graphql',
        originUrl: 'https://countries.trevorblades.com/',
      },
    },
  },
}

export default config
