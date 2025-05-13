'use client';

import { cacheExchange } from '@urql/exchange-graphcache';
import { Client, Provider, fetchExchange } from 'urql';
import { ReactNode, useEffect, useState } from 'react';
import { ClickCapture } from './components/click-capture';

const client = new Client({
  url: '/uncached/graphql',
  fetchOptions: {
    headers: { 'x-defer-demo-delay': '2000' }
  },
  exchanges: [
    cacheExchange({
      keys: {
        Image: (data) => data.src as string,
        CartItem: () => null
      }
    }),
    fetchExchange
  ]
});

export default function RootLayout({ children }: { children: ReactNode }) {
  const [isClient, setIsClient] = useState(false);

  useEffect(() => {
    setIsClient(true);
  }, []);

  if (!isClient) return null;

  return (
    <Provider value={client}>
      <main>{children}</main>
      <ClickCapture />
    </Provider>
  );
}
