import SchemaBuilder from '@pothos/core'
import SimpleObjectsPlugin from '@pothos/plugin-simple-objects'

const builder = new SchemaBuilder({
  plugins: [SimpleObjectsPlugin],
})

const Query = builder.queryType({})

// Fields for testing
builder.queryFields((t) => ({
  noMaxAge: t.int({ nullable: true, resolve: () => null }),
  zeroMaxAge: t.int({ nullable: true, resolve: () => 0 }),
  lowMaxAge: t.int({ nullable: true, resolve: () => 100 }),
  highMaxAge: t.int({ nullable: true, resolve: () => 200 }),
  nested: t.field({ type: Query, nullable: true, resolve: () => null }),
}))

// Medium
const Medium_Paragraph = builder.simpleObject('Medium_Paragraph', {
  fields: (t) => ({
    id: t.id(),
    type: t.string(),
    text: t.string({ nullable: true }),
  }),
})

const Medium_RichText = builder.simpleObject('Medium_RichText', {
  fields: (t) => ({
    paragraphs: t.field({ type: [Medium_Paragraph] }),
  }),
})

const Medium_PostContent = builder.simpleObject('Medium_PostContent', {
  fields: (t) => ({
    bodyModel: t.field({ type: Medium_RichText }),
  }),
})

const Medium_User = builder.simpleObject('Medium_User', {
  fields: (t) => ({
    id: t.id(),
    name: t.string(),
  }),
})

const Medium_Post = builder.simpleObject('Medium_Post', {
  fields: (t) => ({
    id: t.id(),
    creator: t.field({ type: Medium_User }),
    content: t.field({ type: Medium_PostContent }),
  }),
})

const Medium = builder.simpleObject('Medium', {
  fields: (t) => ({
    post: t.field({
      args: { id: t.arg.id({ required: true }) },
      type: Medium_Post,
    }),
  }),
})

builder.queryField('medium', (t) =>
  t.field({
    type: Medium,
    nullable: true,
    resolve: () => ({
      post: {
        id: '42',
        content: {
          bodyModel: {
            paragraphs: [
              { id: '43', type: 'P', text: 'Hello Medium' },
              { id: '44', type: 'IMG' },
            ],
          },
        },
        creator: { id: '45', name: 'Thomas' },
      },
    }),
  }),
)

// Puma
const Puma_Product = builder.simpleObject('Puma_Product', {
  fields: (t) => ({
    id: t.id(),
    name: t.string(),
    price: t.int(),
  }),
})

const Puma = builder.simpleObject('Puma', {
  fields: (t) => ({
    products: t.field({
      type: [Puma_Product],
      nullable: { items: false, list: true },
    }),
  }),
})

builder.queryField('puma', (t) =>
  t.field({
    type: Puma,
    nullable: true,
    resolve: () => ({ products: [{ id: '42', name: 'shoes', price: 4242 }] }),
  }),
)

// Other fields
const Node = builder.simpleInterface('Node', {
  fields: (t) => ({
    id: t.id(),
  }),
})

const Author = builder.simpleObject('Author', {
  interfaces: [Node],
  fields: (t) => ({
    name: t.string(),
  }),
})

const Todo = builder.simpleObject('Todo', {
  interfaces: [Node],
  fields: (t) => ({
    text: t.string(),
    completed: t.boolean({ nullable: true }),
    authors: t.field({
      type: [Author],
      nullable: { items: false, list: true },
    }),
  }),
})

const AUTHOR = { id: '52', name: 'Thomas' }

const TODO = {
  id: '62',
  text: 'Get milk',
  completed: false,
  authors: [AUTHOR],
}

builder.queryFields((t) => ({
  node: t.field({
    args: { id: t.arg.id({ required: true }) },
    type: Node,
    nullable: true,
    resolve: (_, args) =>
      args.id === '52' ? AUTHOR : args.id === '62' ? TODO : null,
  }),
  todo: t.field({
    args: { id: t.arg.id({ required: true }) },
    type: Todo,
    nullable: true,
    resolve: () => TODO,
  }),
  author: t.field({
    args: { id: t.arg.id({ required: true }) },
    type: Author,
    nullable: true,
    resolve: () => AUTHOR,
  }),
}))

export const schema = builder.toSchema()
