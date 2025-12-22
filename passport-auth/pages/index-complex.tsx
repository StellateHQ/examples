import { useLazyQuery, useQuery } from '../utils/useQuery'
import { useState } from 'react'

const isAuthenticatedQuery = /* GraphQL */ `
  {
    me {
      name
    }
  }
`

const loginMutation = /* GraphQL */ `
  mutation ($username: String!, $password: String!) {
    login(username: $username, password: $password)
  }
`

const logoutMutation = /* GraphQL */ `
  mutation {
    logout
  }
`

function AuthQuery(result: ReturnType<typeof useQuery>[0]) {
  if (result.fetching) {
    return (
      <div className='flex items-center justify-center p-4'>
        <div className='h-8 w-8 animate-spin rounded-full border-b-2 border-blue-600'></div>
        <span className='ml-2 text-gray-600'>Loading GraphQL data...</span>
      </div>
    )
  }

  if (result.error) {
    return (
      <div className='mt-6 rounded-lg border border-red-200 bg-red-50 p-4'>
        <div className='mb-2 flex items-center'>
          <svg
            className='mr-2 h-5 w-5 text-red-500'
            fill='currentColor'
            viewBox='0 0 20 20'
          >
            <path
              fillRule='evenodd'
              d='M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7 4a1 1 0 11-2 0 1 1 0 012 0zm-1-9a1 1 0 00-1 1v4a1 1 0 102 0V6a1 1 0 00-1-1z'
              clipRule='evenodd'
            />
          </svg>
          <h3 className='text-lg font-medium text-red-800'>GraphQL Error</h3>
        </div>
        <p className='mb-3 text-sm text-red-600'>{result.error.message}</p>
        {result.error.responseText && (
          <details className='text-xs text-red-500'>
            <summary className='cursor-pointer hover:text-red-700'>
              View full response
            </summary>
            <pre className='mt-2 overflow-x-auto rounded bg-red-100 p-2 text-xs'>
              {result.error.responseText}
            </pre>
          </details>
        )}
      </div>
    )
  }

  return (
    <div className='mt-6 rounded-lg border border-green-200 bg-green-50 p-4'>
      <h3 className='mb-3 text-lg font-medium text-green-800'>
        GraphQL Response
      </h3>
      <pre className='overflow-x-auto rounded bg-green-100 p-3 text-sm text-green-800'>
        {JSON.stringify(result.data, null, 2)}
      </pre>
    </div>
  )
}

export default function Home() {
  const [result, refetch] = useQuery({ query: isAuthenticatedQuery })
  const [, login] = useLazyQuery({ query: loginMutation })
  const [, logout] = useLazyQuery({ query: logoutMutation })
  const [isLoading, setIsLoading] = useState(false)

  if (result.fetching) {
    return (
      <div className='flex min-h-screen items-center justify-center bg-gradient-to-br from-purple-50 to-blue-100'>
        <div className='text-center'>
          <div className='mx-auto h-12 w-12 animate-spin rounded-full border-b-2 border-purple-600'></div>
          <p className='mt-4 text-gray-600'>Loading session...</p>
        </div>
      </div>
    )
  }

  const name = result.data?.me?.name

  const handleLogin = async (event: React.FormEvent<HTMLFormElement>) => {
    event.preventDefault()
    setIsLoading(true)
    try {
      const form = event.target as HTMLFormElement
      await login({
        username: form.username.value,
        password: form.password.value,
      })
      await refetch()
    } finally {
      setIsLoading(false)
    }
  }

  const handleLogout = async () => {
    setIsLoading(true)
    try {
      await logout()
      await refetch()
    } finally {
      setIsLoading(false)
    }
  }

  const handleRefetch = async () => {
    setIsLoading(true)
    try {
      await refetch()
    } finally {
      setIsLoading(false)
    }
  }

  return (
    <div className='min-h-screen bg-gradient-to-br from-purple-50 to-blue-100'>
      <div className='container mx-auto px-4 py-16'>
        <div className='mx-auto max-w-2xl'>
          {/* Header */}
          <div className='mb-12 text-center'>
            <h1 className='mb-4 text-4xl font-bold text-gray-900'>
              Passport.js Demo
            </h1>
            <p className='text-xl text-gray-600'>
              Authentication with GraphQL integration using Passport.js
            </p>
          </div>

          {/* Auth Card */}
          <div className='rounded-xl bg-white p-8 shadow-lg'>
            {name ? (
              <div>
                <div className='mb-6 flex items-center'>
                  <div className='mr-4 flex h-12 w-12 items-center justify-center rounded-full bg-green-100'>
                    <svg
                      className='h-6 w-6 text-green-600'
                      fill='none'
                      stroke='currentColor'
                      viewBox='0 0 24 24'
                    >
                      <path
                        strokeLinecap='round'
                        strokeLinejoin='round'
                        strokeWidth={2}
                        d='M5 13l4 4L19 7'
                      />
                    </svg>
                  </div>
                  <div>
                    <h2 className='text-2xl font-semibold text-gray-900'>
                      Welcome back!
                    </h2>
                    <p className='text-gray-600'>
                      Signed in as{' '}
                      <span className='font-medium text-purple-600'>
                        {name}
                      </span>
                    </p>
                  </div>
                </div>

                <div className='mb-6 flex gap-4'>
                  <button
                    onClick={handleRefetch}
                    disabled={isLoading}
                    className='flex flex-1 items-center justify-center rounded-lg bg-blue-600 px-4 py-3 font-medium text-white transition-colors hover:bg-blue-700 disabled:cursor-not-allowed disabled:opacity-50'
                  >
                    {isLoading ? (
                      <>
                        <div className='mr-2 h-4 w-4 animate-spin rounded-full border-b-2 border-white'></div>
                        Refetching...
                      </>
                    ) : (
                      'Refetch Data'
                    )}
                  </button>
                  <button
                    onClick={handleLogout}
                    disabled={isLoading}
                    className='flex flex-1 items-center justify-center rounded-lg bg-red-600 px-4 py-3 font-medium text-white transition-colors hover:bg-red-700 disabled:cursor-not-allowed disabled:opacity-50'
                  >
                    {isLoading ? (
                      <>
                        <div className='mr-2 h-4 w-4 animate-spin rounded-full border-b-2 border-white'></div>
                        Signing out...
                      </>
                    ) : (
                      'Sign out'
                    )}
                  </button>
                </div>
              </div>
            ) : (
              <div>
                <div className='mb-6 text-center'>
                  <div className='mx-auto mb-4 flex h-12 w-12 items-center justify-center rounded-full bg-purple-100'>
                    <svg
                      className='h-6 w-6 text-purple-600'
                      fill='none'
                      stroke='currentColor'
                      viewBox='0 0 24 24'
                    >
                      <path
                        strokeLinecap='round'
                        strokeLinejoin='round'
                        strokeWidth={2}
                        d='M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z'
                      />
                    </svg>
                  </div>
                  <h2 className='mb-2 text-2xl font-semibold text-gray-900'>
                    Sign in to continue
                  </h2>
                  <p className='text-gray-600'>
                    Use any username and password to sign in
                  </p>
                </div>

                <form onSubmit={handleLogin} className='space-y-4'>
                  <div>
                    <label
                      htmlFor='username'
                      className='mb-2 block text-sm font-medium text-gray-700'
                    >
                      Username
                    </label>
                    <input
                      id='username'
                      name='username'
                      type='text'
                      required
                      className='w-full rounded-lg border border-gray-300 px-3 py-2 focus:border-purple-500 focus:outline-none focus:ring-2 focus:ring-purple-500'
                      placeholder='Enter your username'
                    />
                  </div>
                  <div>
                    <label
                      htmlFor='password'
                      className='mb-2 block text-sm font-medium text-gray-700'
                    >
                      Password
                    </label>
                    <input
                      id='password'
                      name='password'
                      type='password'
                      required
                      className='w-full rounded-lg border border-gray-300 px-3 py-2 focus:border-purple-500 focus:outline-none focus:ring-2 focus:ring-purple-500'
                      placeholder='Enter your password'
                    />
                  </div>
                  <button
                    type='submit'
                    disabled={isLoading}
                    className='flex w-full items-center justify-center rounded-lg bg-purple-600 px-4 py-3 font-medium text-white transition-colors hover:bg-purple-700 disabled:cursor-not-allowed disabled:opacity-50'
                  >
                    {isLoading ? (
                      <>
                        <div className='mr-2 h-4 w-4 animate-spin rounded-full border-b-2 border-white'></div>
                        Signing in...
                      </>
                    ) : (
                      'Sign in'
                    )}
                  </button>
                </form>
              </div>
            )}

            {/* GraphQL Query Section */}
            <div className='mt-8 border-t border-gray-200 pt-8'>
              <h3 className='mb-4 text-lg font-semibold text-gray-900'>
                GraphQL API Test
              </h3>
              <p className='mb-4 text-gray-600'>
                This section demonstrates GraphQL queries with Passport.js
                authentication.
              </p>
              <AuthQuery {...result} />
            </div>
          </div>

          {/* Footer */}
          <div className='mt-12 text-center text-gray-500'>
            <p>
              Built with{' '}
              <a
                href='https://www.passportjs.org'
                className='text-purple-600 hover:text-purple-800'
                target='_blank'
                rel='noopener noreferrer'
              >
                Passport.js
              </a>{' '}
              and{' '}
              <a
                href='https://nextjs.org'
                className='text-purple-600 hover:text-purple-800'
                target='_blank'
                rel='noopener noreferrer'
              >
                Next.js 16
              </a>
            </p>
          </div>
        </div>
      </div>
    </div>
  )
}
