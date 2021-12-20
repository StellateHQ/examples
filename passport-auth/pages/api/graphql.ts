/**
 * This endpoint provides a minimal GraphQL API with authentication using
 * `passport`.
 */

import { envelop, useSchema } from '@envelop/core'
import session from 'express-session'
import { createTypesFactory, buildGraphQLSchema } from 'gqtx'
import { GraphQLError } from 'graphql'
import { NextApiRequest, NextApiResponse } from 'next'
import nextConnect from 'next-connect'
import passport from 'passport'
import { Strategy as LocalStrategy } from 'passport-local'

/**
 * Setup passport
 */

declare global {
  namespace Express {
    // tslint:disable-next-line:no-empty-interface
    interface AuthInfo {}
    // tslint:disable-next-line:no-empty-interface
    interface User {
      name?: string
    }
  }
}

passport.use(
  new LocalStrategy((username, password, done) => {
    // For the sake of example, we don't validate credentials, i.e. login works
    // with every username-password-combination.
    const user: Express.User = { name: username }
    return done(null, user)
  }),
)

passport.serializeUser((user, done) => {
  done(null, user)
})

passport.deserializeUser((user, done) => {
  done(null, user)
})

type ReqWithAuth = NextApiRequest & Express.Request

function authenticate(req: ReqWithAuth, res: NextApiResponse) {
  return new Promise<Express.User>((resolve, reject) => {
    passport.authenticate('local', (error, token) => {
      if (error) {
        reject(error)
      } else {
        resolve(token)
      }
    })(req, res)
  })
}

function login(req: ReqWithAuth, user: Express.User) {
  return new Promise<boolean>((resolve, reject) => {
    req.login(user, (error) => {
      if (error) return reject(error)
      resolve(true)
    })
  })
}

function logout(req: ReqWithAuth, res: NextApiResponse) {
  return new Promise<boolean>((resolve, reject) => {
    req.logout()
    req.session.destroy((error) => {
      if (error) return reject(error)
      res.setHeader(
        'Set-Cookie',
        'connect.sid=; Path=/; HttpOnly; Expires=Thu, 01 Jan 1970 00:00:00 GMT',
      )
      resolve(true)
    })
  })
}

/**
 * Create the GraphQL schema
 */

const t = createTypesFactory<{ req: ReqWithAuth; res: NextApiResponse }>()

const User = t.objectType<Express.User>({
  name: 'User',
  fields: () => [t.field({ name: 'name', type: t.String })],
})

const Query = t.queryType({
  fields: () => [
    t.field({
      name: 'me',
      type: User,
      resolve(_root, _args, ctx): Express.User {
        const user = ctx.req.user
        if (!user) {
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
        return user
      },
    }),
  ],
})

const Mutation = t.mutationType({
  fields: () => [
    t.field({
      name: 'login',
      type: t.Boolean,
      args: {
        username: t.arg(t.NonNullInput(t.String)),
        password: t.arg(t.NonNullInput(t.String)),
      },
      async resolve(_, args, ctx) {
        ctx.req.body.username = args.username
        ctx.req.body.password = args.password
        const user = await authenticate(ctx.req, ctx.res)
        return login(ctx.req, user)
      },
    }),
    t.field({
      name: 'logout',
      type: t.Boolean,
      resolve(_root, _args, ctx) {
        return logout(ctx.req, ctx.res)
      },
    }),
  ],
})

const schema = buildGraphQLSchema({ query: Query, mutation: Mutation })

/**
 * Setup an endpoint that handles GraphQL request using `envelop`
 */

const getEnveloped = envelop({
  plugins: [useSchema(schema)],
})

export default nextConnect()
  .use(
    session({
      secret: process.env.SESSION_SECRET,
      resave: false,
      saveUninitialized: false,
    }),
  )
  .use(passport.initialize())
  .use(passport.session())
  .post(async (req: ReqWithAuth, res: NextApiResponse) => {
    if (req.method.toUpperCase() !== 'POST') {
      return res
        .setHeader('Allow', 'POST')
        .status(405)
        .send('Method Not Allowed')
    }

    try {
      const { parse, validate, contextFactory, execute, schema } = getEnveloped(
        { req, res },
      )

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
    } catch (error) {
      if (error instanceof GraphQLError === false) {
        error = new GraphQLError(error.message)
      }
      res.status(500).json({ errors: [error] })
    }
  })
