'use server';

import { USER_COOKIE, revalidateCarts, sql } from 'lib/utils';
import { cookies } from 'next/headers';
import { v4 as uuidv4 } from 'uuid';

export async function addToCart(_prevState: any, productId: string) {
  const userId = (await cookies()).get(USER_COOKIE)?.value;
  if (!userId) {
    return 'Must be logged in to add products to cart';
  }

  try {
    const existingCartItem = await sql<{ quantity: number }>(
      `
        SELECT cartitems.quantity
        FROM users_cartitems
        JOIN cartitems ON cartitems.id = users_cartitems.cartitem_id
        WHERE users_cartitems.user_id = ?
          AND cartitems.product_id = ?
      `,
      [userId, productId]
    );
    if (existingCartItem.length > 0) {
      await _updateItemQuantity(userId, productId, existingCartItem[0]!.quantity + 1);
    } else {
      const cartItemId = uuidv4();
      await sql('INSERT INTO cartitems (id, product_id, quantity) VALUES (?, ?, ?)', [
        cartItemId,
        productId,
        1
      ]);
      await sql('INSERT INTO users_cartitems (user_id, cartitem_id) VALUES (?, ?)', [
        userId,
        cartItemId
      ]);
    }

    revalidateCarts();
  } catch (error) {
    console.error(error);
    return 'Error adding item to cart';
  }
}

async function _removeFromCart(userId: string, productId: string) {
  return sql(
    `
      DELETE FROM users_cartitems
      WHERE users_cartitems.user_id = ?
        AND users_cartitems.cartitem_id IN (
          SELECT cartitems.id
          FROM users_cartitems
          JOIN cartitems ON cartitems.id = users_cartitems.cartitem_id
          WHERE users_cartitems.user_id = ?
            AND cartitems.product_id = ?
        )
    `,
    [userId, userId, productId]
  );
}

export async function removeFromCart(_prevState: any, productId: string) {
  const userId = (await cookies()).get(USER_COOKIE)?.value;
  if (!userId) {
    return 'Must be logged in to remove products from cart';
  }

  try {
    await _removeFromCart(userId, productId);
    revalidateCarts();
  } catch (error) {
    console.error(error);
    return 'Error removing item from cart';
  }
}

async function _updateItemQuantity(userId: string, productId: string, quantity: number) {
  return sql(
    `
      UPDATE cartitems
      SET quantity = ?
      WHERE id IN (
        SELECT cartitems.id
        FROM cartitems
        JOIN users_cartitems ON users_cartitems.cartitem_id = cartitems.id
        WHERE users_cartitems.user_id = ?
          AND cartitems.product_id = ?
      )
    `,
    [quantity, userId, productId]
  );
}

export async function updateItemQuantity(
  _prevState: any,
  payload: {
    productId: string;
    quantity: number;
  }
) {
  const userId = (await cookies()).get(USER_COOKIE)?.value;
  if (!userId) {
    return 'Must be logged in to edit cart';
  }

  const { productId, quantity } = payload;

  try {
    if (quantity === 0) {
      await _removeFromCart(userId, productId);
    } else {
      _updateItemQuantity(userId, productId, quantity);
    }
    revalidateCarts();
  } catch (error) {
    console.error(error);
    return 'Error updating item quantity';
  }
}
