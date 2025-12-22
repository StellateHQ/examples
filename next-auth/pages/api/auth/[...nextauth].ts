import NextAuth from 'next-auth'
import CredentialsProvider from 'next-auth/providers/credentials'

export const authOptions = {
  providers: [
    // For demonstration we use credentials auth; any combination is accepted
    CredentialsProvider({
      name: 'Credentials',
      credentials: {
        username: { label: 'Username', type: 'text' },
        password: { label: 'Password', type: 'password' },
      },
      async authorize(credentials) {
        if (!credentials?.username) {
          return null
        }
        // NextAuth's User type requires at least an `id` field.
        return {
          id: credentials.username,
          name: credentials.username,
        }
      },
    }),
  ],
  secret: process.env.NEXTAUTH_SECRET,
  // only in production: set cookie domain for your subdomains
  ...(process.env.NEXT_PUBLIC_VERCEL_ENV === 'production'
    ? {
        cookies: {
          sessionToken: {
            name: '__Secure-next-auth.session-token',
            options: {
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
}

export default NextAuth(authOptions)
