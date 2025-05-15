import { makeExecutableSchema } from '@graphql-tools/schema'
import gql from 'graphql-tag'
import { createState } from '../utils'
import type { AdminContext } from '../types'

const typeDefs = gql`
  type Query {
    _empty: String
  }

  type Mutation {
    createEndpoint(slug: String!): String!
  }
`

const resolvers = {
  Mutation: {
    createEndpoint: async (
      _: any,
      { slug }: { slug: string },
      context: AdminContext,
    ) => {
      if (await context.env.STATE.get(slug)) {
        throw new Error(`Endpoint with slug ${slug} already taken`)
      }
      await context.env.STATE.set(slug, JSON.stringify(createState()))
      return slug
    },
  },
}

export const adminSchema = makeExecutableSchema({ typeDefs, resolvers })
