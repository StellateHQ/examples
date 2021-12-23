import { useSession, signIn, signOut } from 'next-auth/react'
import { useQuery } from '../utils/useQuery'

const query = /* GraphQL */ `
  {
    me {
      name
    }
  }
`

function AuthQuery() {
  const [result, refetch] = useQuery({ query })

  if (result.fetching) {
    return <p>Loading...</p>
  }
  if (result.error) {
    return (
      <>
        <h1>Error</h1>
        <pre>{JSON.stringify(result.error, null, 2)}</pre>;
      </>
    )
  }
  return (
    <>
      <button
        onClick={() => {
          refetch()
        }}
      >
        Refetch
      </button>
      <h1>Data</h1>
      <pre>{JSON.stringify(result.data, null, 2)}</pre>
    </>
  )
}

export default function Home() {
  const { status, data } = useSession()
  if (status === 'loading') {
    return <p>Loading...</p>
  }
  if (status === 'authenticated') {
    return (
      <>
        Signed in as {data.user.name} <br />
        <button
          onClick={() => {
            signOut()
          }}
        >
          Sign out
        </button>
        <br />
        <AuthQuery />
      </>
    )
  }
  return (
    <>
      Not signed in <br />
      <button
        onClick={() => {
          signIn()
        }}
      >
        Sign in
      </button>
      <br />
      <AuthQuery />
    </>
  )
}
