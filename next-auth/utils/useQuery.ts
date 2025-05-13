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
    async (variables: Record<string, any> = {}) => {
      setResult((r) => ({ ...r, fetching: true, error: null }))
      try {
        const res = await fetch(process.env.NEXT_PUBLIC_API_ENDPOINT!, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          credentials: 'include',
          body: JSON.stringify({
            query: body.query,
            variables: { ...body.variables, ...variables },
          }),
        })

        const text = await res.text()
        console.log('📬 HTTP', res.status, res.statusText, '→', text)

        if (!res.ok) {
          // HTTP-level error (404, 500, etc)
          setResult({
            fetching: false,
            error: {
              message: `HTTP error ${res.status}: ${res.statusText}`,
              responseText: text,
            },
            data: null,
          })
          return
        }

        let json: any
        try {
          json = JSON.parse(text)
        } catch {
          setResult({
            fetching: false,
            error: { message: 'Invalid JSON response', responseText: text },
            data: null,
          })
          return
        }

        const graphQLError = json.errors?.[0]
        setResult({
          fetching: false,
          error: graphQLError
            ? {
                message: graphQLError.message,
                graphQLErrors: json.errors,
              }
            : null,
          data: json.data ?? null,
        })
      } catch (err: any) {
        // Network or other
        setResult({
          fetching: false,
          error: { message: err.message },
          data: null,
        })
      }
    },
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
