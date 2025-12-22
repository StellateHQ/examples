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
    interface AuthInfo {}
    interface User {
      name?: string
    }
  }
}

passport.use(
  new LocalStrategy((username, password, done) => {
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
    // Set timeout to prevent hanging
    const timeoutId = setTimeout(() => {
      reject(new Error('Logout timeout'))
    }, 5000) // 5 second timeout

    try {
      // pass a callback to logout(), per latest TS definitions
      req.logout((err: any) => {
        clearTimeout(timeoutId)
        if (err) {
          return reject(err)
        }

        // Clear the session cookie immediately
        res.setHeader(
          'Set-Cookie',
          'connect.sid=; Path=/; HttpOnly; Expires=Thu, 01 Jan 1970 00:00:00 GMT; SameSite=lax',
        )

        // Try to destroy session but don't wait for it if it hangs
        if (req.session && req.session.destroy) {
          req.session.destroy((error) => {
            // Ignore destroy errors, we've already cleared the cookie
            resolve(true)
          })
        } else {
          resolve(true)
        }
      })
    } catch (error) {
      clearTimeout(timeoutId)
      reject(error)
    }
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
            undefined,
            undefined,
            undefined,
            undefined,
            undefined,
            {
              statusCode: 401,
            },
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

const handler = nextConnect()
  .use(
    session({
      secret: process.env.SESSION_SECRET!,
      resave: false,
      saveUninitialized: false,
    }),
  )
  .use(passport.initialize())
  .use(passport.session())
  .get((req: NextApiRequest, res: NextApiResponse) => {
    // Handle GET requests - return GraphQL playground or info
    res.setHeader('Content-Type', 'text/html')
    res.status(200).send(`
      <!DOCTYPE html>
      <html>
        <head>
          <title>GraphQL API</title>
          <style>
            body { font-family: Arial, sans-serif; margin: 40px; background: #f5f5f5; }
            .container { background: white; padding: 40px; border-radius: 8px; box-shadow: 0 2px 10px rgba(0,0,0,0.1); }
            h1 { color: #333; }
            p { color: #666; line-height: 1.6; }
            code { background: #f0f0f0; padding: 2px 8px; border-radius: 4px; }
          </style>
        </head>
        <body>
          <div class="container">
            <h1>🚀 GraphQL API with Passport.js</h1>
            <p>This GraphQL endpoint supports authentication using Passport.js local strategy.</p>
            <p><strong>Endpoint:</strong> <code>POST /api/graphql</code></p>
            <p><strong>Available queries:</strong></p>
            <ul>
              <li><code>query { me { name } }</code> - Get authenticated user</li>
            </ul>
            <p><strong>Available mutations:</strong></p>
            <ul>
              <li><code>mutation { login(username: "test", password: "test") }</code> - Login</li>
              <li><code>mutation { logout }</code> - Logout</li>
            </ul>
            <p><a href="/">← Back to main application</a></p>
          </div>
        </body>
      </html>
    `)
  })
  .options((req: NextApiRequest, res: NextApiResponse) => {
    // Handle CORS preflight
    res.setHeader('Access-Control-Allow-Origin', '*')
    res.setHeader('Access-Control-Allow-Methods', 'GET, POST, OPTIONS')
    res.setHeader('Access-Control-Allow-Headers', 'Content-Type, Authorization')
    res.status(200).end()
  })
  .post(async (req: ReqWithAuth, res: NextApiResponse) => {
    // Add CORS headers
    res.setHeader('Access-Control-Allow-Origin', '*')
    res.setHeader('Access-Control-Allow-Methods', 'GET, POST, OPTIONS')
    res.setHeader('Access-Control-Allow-Headers', 'Content-Type, Authorization')

    try {
      const { parse, validate, contextFactory, execute, schema } = getEnveloped(
        { req, res },
      )
      const { query, variables } = req.body
      const document = parse(query)
      const validationErrors = validate(schema, document)

      if (validationErrors.length > 0) {
        res.status(400).json({ errors: validationErrors })
        return
      }

      const contextValue = await contextFactory()
      const result = await execute({
        document,
        schema,
        variableValues: variables,
        contextValue,
      })

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
      if (!(error instanceof GraphQLError)) {
        error = new GraphQLError((error as Error).message)
      }
      res.status(500).json({ errors: [error] })
    }
  })

export default handler
