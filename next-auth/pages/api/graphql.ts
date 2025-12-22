/**
 * This endpoint provides a minimal GraphQL API with authentication using
 * `next-auth`.
 */

import { envelop, useSchema } from '@envelop/core'
import { createTypesFactory, buildGraphQLSchema } from 'gqtx'
import { GraphQLError } from 'graphql'
import { NextApiRequest, NextApiResponse } from 'next'
import { getServerSession } from 'next-auth/next'
import { authOptions } from './auth/[...nextauth]'

/**
 * Create the GraphQL schema
 */

const t = createTypesFactory<{ req: NextApiRequest; res: NextApiResponse }>()

type UserType = { name?: string }

const User = t.objectType<UserType>({
  name: 'User',
  fields: () => [t.field({ name: 'name', type: t.String })],
})

const Query = t.queryType({
  fields: () => [
    t.field({
      name: 'me',
      type: User,
      async resolve(_root, args, ctx): Promise<UserType> {
        const session = (await getServerSession(
          ctx.req,
          ctx.res,
          authOptions,
        )) as any
        if (!session) {
          throw new GraphQLError(
            'Not authenticated',
            undefined, // nodes
            undefined, // source
            undefined, // positions
            undefined, // path
            undefined, // originalError
            { statusCode: 401 },
          )
        }
        return { name: session.user.name }
      },
    }),
  ],
})

const schema = buildGraphQLSchema({ query: Query })

/**
 * Setup an endpoint that handles GraphQL request using `envelop`
 */

const getEnveloped = envelop({
  plugins: [useSchema(schema)],
})

export default async function handler(
  req: NextApiRequest,
  res: NextApiResponse,
) {
  if (req.method.toUpperCase() !== 'POST') {
    return res.setHeader('Allow', 'POST').status(405).send('Method Not Allowed')
  }

  try {
    const { parse, validate, contextFactory, execute, schema } = getEnveloped({
      req,
      res,
    })

    const { query, variables } = req.body
    const document = parse(query)
    const validationErrors = validate(schema, document)

    if (validationErrors.length > 0) {
      return res.status(400).json({ errors: validationErrors })
    }

    const contextValue = await contextFactory()
    const result = await execute({
      document,
      schema,
      variableValues: variables,
      contextValue,
    })

    // It's important to use semantically correct status codes! A response
    // with a 200 status code is considered cacheable by fastly. If it
    // contains errors, GraphCDN will still set the `Cache-Control` header
    // accordingly (`private, no-store`), but Fastly might disable caching
    // for this particular query for up to two minutes if the status code
    // of the response is considered cacheable. (See https://developer.fastly.com/learning/concepts/request-collapsing/#hit-for-pass)
    const statusCode = result.errors
      ? result.errors.reduce((code, error) => {
          return Math.max(
            code,
            typeof error.extensions.statusCode === 'number'
              ? error.extensions.statusCode
              : 0,
          )
        }, 200)
      : 200
    res.status(statusCode).json(result)
  } catch (err) {
    if (err instanceof GraphQLError === false) {
      err = new GraphQLError(err.message)
    }
    res.status(500).json({ errors: [err] })
  }
}
