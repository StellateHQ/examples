import { StorageAdapter, cacheExchange } from '@urql/exchange-graphcache';
import { kv } from '@vercel/kv';
import { TadaDocumentNode } from 'gql.tada';
import { buildClientSchema, getNamedType, TypeInfo, visit, visitWithTypeInfo } from 'graphql';
import { revalidateTag } from 'next/cache';
import { cookies } from 'next/headers';
import { cache } from 'react';
import { AnyVariables, createClient, fetchExchange } from 'urql';
import introspection from '../introspection.json';
import { USER_COOKIE } from './utils';

const SCHEMA = buildClientSchema(introspection as any);

function makeTag(type: string): string {
  return `GraphQLType:${type}`;
}

const makeStorage = cache(
  (
    userId: string | undefined
  ):
    | {
        adapter: StorageAdapter;
        waitForCacheHydration: () => Promise<void>;
        waitForCacheWrite: () => Promise<void>;
      }
    | undefined => {
    if (!userId) return undefined;

    const store = {};

    let onHydrated: () => void;
    const hydrationPromise = new Promise<void>((res) => {
      onHydrated = res;
    });

    let onWritten: () => void;
    const writePromise = new Promise<void>((res) => {
      onWritten = res;
    });

    const adapter: StorageAdapter = {
      async writeData(delta) {
        Object.assign(store, delta);
        if (Object.keys(delta).length > 0) {
          await kv.set(userId, store);
          onWritten();
        }
      },
      async readData() {
        const kvValue = (await kv.get(userId)) ?? null;
        Object.assign(store, kvValue);
        return store;
      },
      onCacheHydrated() {
        onHydrated();
      }
    };

    return {
      adapter,
      waitForCacheHydration: async () => await hydrationPromise,
      waitForCacheWrite: async () => await writePromise
    };
  }
);

const getClient = cache((storage: StorageAdapter | undefined) => {
  return createClient({
    url: 'http://localhost:8787/graphql',
    exchanges: [
      cacheExchange({
        keys: {
          Image: (data) => data.src as string,
          CartItem: () => null
        },
        storage
      }),
      fetchExchange
    ]
  });
});

// eslint-disable-next-line no-unused-vars
async function gql_depr<Data, Variables extends AnyVariables>(
  query: TadaDocumentNode<Data, Variables>,
  variables: Variables
): Promise<Data> {
  const userId = cookies().get(USER_COOKIE)?.value;
  const storage = makeStorage(userId);
  const client = getClient(storage?.adapter);

  await storage?.waitForCacheHydration();

  let isMutation = false;
  const typeTags = new Set<string>();

  const typeInfo = new TypeInfo(SCHEMA);
  visit(
    query,
    visitWithTypeInfo(typeInfo, {
      OperationDefinition(node) {
        isMutation = node.operation === 'mutation';
      },
      Field(node) {
        if (node.selectionSet?.selections.length) {
          const type = getNamedType(typeInfo.getFieldDef()?.type)?.name;
          if (type) typeTags.add(makeTag(type));
        }
      }
    })
  );

  const response = await client[isMutation ? 'mutation' : 'query'](query, variables, {
    fetchOptions: {
      headers: {
        ...(userId ? { 'x-user-id': userId } : {}),

        // Avoid artificial delay
        'x-stellate-e2e': 'true'
      },

      // Add tags to queries
      ...(isMutation ? {} : { next: { tags: isMutation ? [] : Array.from(typeTags) } })
    }
  });

  if (response.error) {
    throw response.error.message;
  }
  if (!response.data) {
    throw 'Missing data';
  }

  if (isMutation) {
    await storage?.waitForCacheWrite();
    typeTags.forEach((typeTag) => revalidateTag(typeTag));
  }

  return response.data;
}
