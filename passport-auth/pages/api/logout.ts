import { NextApiRequest, NextApiResponse } from 'next'
import nextConnect from 'next-connect'
import session from 'express-session'
import passport from 'passport'
import { Strategy as LocalStrategy } from 'passport-local'

// Setup passport (same as in graphql.ts)
passport.use(
  new LocalStrategy((username, password, done) => {
    const user = { name: username }
    return done(null, user)
  }),
)

passport.serializeUser((user, done) => {
  done(null, user)
})

passport.deserializeUser((user, done) => {
  done(null, user as any)
})

type ReqWithAuth = NextApiRequest & Express.Request

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
  .post(async (req: ReqWithAuth, res: NextApiResponse) => {
    try {
      // Simple logout - just clear the cookie
      res.setHeader(
        'Set-Cookie',
        'connect.sid=; Path=/; HttpOnly; Expires=Thu, 01 Jan 1970 00:00:00 GMT; SameSite=lax',
      )

      // Try to logout with passport but don't wait too long
      if (req.logout) {
        req.logout(() => {
          // Callback called but we don't wait for session destroy
        })
      }

      res.status(200).json({ success: true })
    } catch (error) {
      // Even on error, we've cleared the cookie
      res
        .status(200)
        .json({ success: true, warning: 'Logout completed with warnings' })
    }
  })

export default handler
