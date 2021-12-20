import { useLazyQuery, useQuery } from '../utils/useQuery'

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
  if (result.error) {
    return (
      <>
        <h1>Error</h1>
        <pre>{JSON.stringify(result.error, null, 2)}</pre>
      </>
    )
  }
  return (
    <>
      <h1>Data</h1>
      <pre>{JSON.stringify(result.data, null, 2)}</pre>
    </>
  )
}

export default function Home() {
  // We use hand-crafted hooks for this simple example, this can easily be
  // replaced with any GraphQL client like `urql` or `apollo`.
  const [result, refetch] = useQuery({ query: isAuthenticatedQuery })
  const [, login] = useLazyQuery({ query: loginMutation })
  const [, logout] = useLazyQuery({ query: logoutMutation })

  if (result.fetching) {
    return <p>Loading...</p>
  }

  const name = result.data?.me?.name
  if (name) {
    return (
      <>
        Signed in as {name} <br />
        <button
          onClick={async () => {
            await refetch()
          }}
        >
          Refetch
        </button>
        <br />
        <button
          onClick={async () => {
            await logout()
            await refetch()
          }}
        >
          Sign out
        </button>
        <AuthQuery {...result} />
      </>
    )
  }

  return (
    <>
      Not signed in <br />
      <form
        onSubmit={async (event) => {
          event.preventDefault()
          const form = event.target as HTMLFormElement
          await login({
            username: form.username.value,
            password: form.password.value,
          })
          await refetch()
        }}
      >
        <input name='username' placeholder='username' />
        <input name='password' placeholder='password' type='password' />
        <button type='submit'>Sign in</button>
        <AuthQuery {...result} />
      </form>
    </>
  )
}
