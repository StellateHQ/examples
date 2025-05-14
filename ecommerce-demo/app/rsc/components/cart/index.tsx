import { cookies } from 'next/headers';
import CartModal, { CartData } from './modal';
import { USER_COOKIE, sql } from 'lib/utils';

export default async function Cart() {
  const userId = cookies().get(USER_COOKIE)?.value;
  const [total, items] = userId
    ? await Promise.all([
        sql<{ total: number | null }>(
          `
            SELECT SUM(cartitems.quantity * products.price) AS total
            FROM users_cartitems
            JOIN cartitems ON cartitems.id = users_cartitems.cartitem_id
            JOIN products ON products.id = cartitems.product_id
            WHERE users_cartitems.user_id = ?
          `,
          [userId]
        ),
        sql<CartData['items'][number]>(
          `
            SELECT cartitems.quantity, products.id, products.slug, products.name, products.price, images.src, images.alt
            FROM users_cartitems
            JOIN cartitems ON cartitems.id = users_cartitems.cartitem_id
            JOIN products ON products.id = cartitems.product_id
            JOIN images ON images.src = products.cover_image_src
            WHERE users_cartitems.user_id = ?
          `,
          [userId]
        )
      ])
    : [];

  return <CartModal cart={items ? { totalPrice: total?.[0]?.total ?? 0, items } : undefined} />;
}
