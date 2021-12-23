import { useCallback, useEffect, useState } from 'react'

type RequestState = {
  fetching: boolean
  error: {
    message: string
    graphQLErrors?: any[]
    responseText?: string
  } | null
  data: Record<string, any> | null
}

export function useLazyQuery(body: {
  query: string
  variables?: Record<string, any>
}) {
  const [result, setResult] = useState<RequestState>({
    fetching: true,
    error: null,
    data: null,
  })

  const execute = useCallback(
    (variables: Record<string, any> = {}) =>
      fetch(process.env.NEXT_PUBLIC_API_ENDPOINT, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          query: body.query,
          variables: { ...body.variables, ...variables },
        }),
        // It's crucial to include credentials when sending the request to
        // GraphCDN. Since it's on a different domain, the session cookie
        // won't be sent with the request otherwise.
        credentials: 'include',
      })
        .then((res) => res.text())
        .then(async (responseText) => {
          let json
          try {
            json = JSON.parse(responseText)
          } catch {
            setResult({
              fetching: false,
              error: { message: 'Cannot parse response as JSON', responseText },
              data: null,
            })
            return
          }

          const graphQLError = json?.errors?.[0]
          const message = graphQLError?.message ?? 'Unknown GraphQL error'
          setResult({
            fetching: false,
            error: graphQLError
              ? { message, graphQLErrors: json.errors }
              : null,
            data: json?.data ?? null,
          })
        })
        .catch((error) => {
          setResult({ fetching: false, error, data: null })
        }),
    [body.query, body.variables],
  )

  return [result, execute] as const
}

export function useQuery(body: {
  query: string
  variables?: Record<string, any>
}) {
  const [result, execute] = useLazyQuery(body)

  useEffect(() => {
    execute()
  }, [execute])

  return [result, execute] as const
}
