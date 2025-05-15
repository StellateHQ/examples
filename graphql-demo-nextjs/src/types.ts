export type User = { id: string; name: string }
export type Todo = {
  id: string
  title: string
  createdAt: string
  user: string
}
export type State = { users: User[]; todos: Todo[] }
export type Resolver<C = any, V = any, A = {}, R = any> = (
  root: R,
  args: A,
  context: C,
) => V | Promise<V>

// Admin context
export type AdminContext = { env: { STATE: import('@upstash/redis').Redis } }
// Endpoint context
export type EndpointContext = { state: State; setState: (s: State) => void }
