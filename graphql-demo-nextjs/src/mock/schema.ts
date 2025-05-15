import { makeExecutableSchema } from '@graphql-tools/schema'
import gql from 'graphql-tag'
import type { EndpointContext } from '../types'

const typeDefs = gql`
  type User {
    id: ID!
    name: String!
    todos: [Todo!]!
  }
  type Todo {
    id: ID!
    title: String!
    createdAt: String!
    user: User
  }
  type Query {
    user(id: ID!): User!
    users: [User!]!
    todo(id: ID!): Todo!
    todos: [Todo!]!
  }
  type Mutation {
    addUser(name: String!): User!
    updateUser(id: ID!, name: String!): User!
    deleteUser(id: ID!): User!
    addTodo(userId: ID!, title: String!): Todo!
    updateTodo(id: ID!, title: String!): Todo!
    deleteTodo(id: ID!): Todo!
  }
`

const resolvers = {
  User: {
    todos: (root: any, _args: any, ctx: EndpointContext) =>
      ctx.state.todos.filter((t) => t.user === root.id),
  },
  Todo: {
    user: (root: any, _args: any, ctx: EndpointContext) =>
      ctx.state.users.find((u) => u.id === root.user)!,
  },
  Query: {
    users: (_: any, __: any, ctx: EndpointContext) => ctx.state.users,
    user: (_: any, { id }: { id: string }, ctx: EndpointContext) =>
      ctx.state.users.find((u) => u.id === id)!,
    todos: (_: any, __: any, ctx: EndpointContext) => ctx.state.todos,
    todo: (_: any, { id }: { id: string }, ctx: EndpointContext) =>
      ctx.state.todos.find((t) => t.id === id)!,
  },
  Mutation: {
    addUser: (_: any, { name }: { name: string }, ctx: EndpointContext) => {
      const newUser = { id: String(ctx.state.users.length + 1), name }
      ctx.setState({ ...ctx.state, users: [...ctx.state.users, newUser] })
      return newUser
    },
    updateUser: (
      _: any,
      { id, name }: { id: string; name: string },
      ctx: EndpointContext,
    ) => {
      const users = ctx.state.users.map((u) =>
        u.id === id ? { ...u, name } : u,
      )
      ctx.setState({ ...ctx.state, users })
      return users.find((u) => u.id === id)!
    },
    deleteUser: (_: any, { id }: { id: string }, ctx: EndpointContext) => {
      const user = ctx.state.users.find((u) => u.id === id)!
      ctx.setState({
        ...ctx.state,
        users: ctx.state.users.filter((u) => u.id !== id),
      })
      return user
    },
    addTodo: (
      _: any,
      { userId, title }: { userId: string; title: string },
      ctx: EndpointContext,
    ) => {
      const newTodo = {
        id: String(ctx.state.todos.length + 1),
        title,
        user: userId,
        createdAt: new Date().toISOString(),
      }
      ctx.setState({ ...ctx.state, todos: [...ctx.state.todos, newTodo] })
      return newTodo
    },
    updateTodo: (
      _: any,
      { id, title }: { id: string; title: string },
      ctx: EndpointContext,
    ) => {
      const todos = ctx.state.todos.map((t) =>
        t.id === id ? { ...t, title } : t,
      )
      ctx.setState({ ...ctx.state, todos })
      return todos.find((t) => t.id === id)!
    },
    deleteTodo: (_: any, { id }: { id: string }, ctx: EndpointContext) => {
      const todo = ctx.state.todos.find((t) => t.id === id)!
      ctx.setState({
        ...ctx.state,
        todos: ctx.state.todos.filter((t) => t.id !== id),
      })
      return todo
    },
  },
}

export const userSchema = makeExecutableSchema({ typeDefs, resolvers })
