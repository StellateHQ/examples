import NextAuth from 'next-auth'
import CredentialsProvider from 'next-auth/providers/credentials'

export default NextAuth({
  providers: [
    // For demonstration we use authentication with credentials, where any
    // username-password-combination is valid. We store the passed username
    // in the next-auth session. However, using any other provider should work
    // just as well.
    CredentialsProvider({
      name: 'Credentials',
      credentials: {
        username: { label: 'Username', type: 'text' },
        password: { label: 'Password', type: 'password' },
      },
      async authorize(credentials) {
        return { name: credentials.username }
      },
    }),
  ],
  secret: process.env.SESSION_SECRET,
  ...(process.env.NEXT_PUBLIC_VERCEL_ENV === 'production'
    ? {
        cookies: {
          sessionToken: {
            name: '__Secure-next-auth.session-token',
            options: {
              // The default would be the exact domain. We also want to allow passing
              // credentials with requests to the `graphcdn` subdomain, so we make
              // this cookie readable for all subdomains.
              // Note that this is only relevant for the production deployment. When
              // developing on localhost, no domain needs to be set.
              domain: `.${process.env.NEXTAUTH_URL}`,
              httpOnly: true,
              path: '/',
              sameSite: 'lax',
              secure: true,
            },
          },
        },
      }
    : {}),
})
