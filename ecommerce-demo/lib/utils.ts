import { revalidateTag } from 'next/cache';
import { cookies } from 'next/headers';

export const USER_COOKIE = 'user-id';

const TAG = {
  CART: 'cart',
  ORDER: 'order'
} as const;

export const revalidateCarts = () => revalidateTag(TAG.CART);

export const revalidateOrders = () => revalidateTag(TAG.ORDER);

export async function sql<Result extends Record<string, unknown> = Record<string, unknown>>(
  preparedStatement: string,
  parameters: any[] = [],
  delay?: number
): Promise<Result[]> {
  if (delay) await new Promise((r) => setTimeout(r, delay));

  const tags: string[] = [];
  if (preparedStatement.trim().startsWith('SELECT')) {
    if (
      preparedStatement.includes('FROM cartitems') ||
      preparedStatement.includes('FROM users_cartitems') ||
      preparedStatement.includes('FROM orders_cartitems') ||
      preparedStatement.includes('JOIN cartitems') ||
      preparedStatement.includes('JOIN users_cartitems') ||
      preparedStatement.includes('JOIN orders_cartitems')
    )
      tags.push(TAG.CART);

    if (
      preparedStatement.includes('FROM orders') ||
      preparedStatement.includes('FROM orders_cartitems') ||
      preparedStatement.includes('JOIN orders') ||
      preparedStatement.includes('JOIN orders_cartitems')
    )
      tags.push(TAG.ORDER);
  }

  const userId = cookies().get(USER_COOKIE)?.value;
  const response = await fetch('http://localhost:8787/sql', {
    method: 'POST',
    headers: {
      'content-type': 'application/json',
      ...(userId ? { 'x-user-id': userId } : {})
    },
    body: JSON.stringify({ preparedStatement, parameters }),
    next: { tags }
  });

  if (!response.ok) {
    throw await response.text();
  }

  return response.json();
}
