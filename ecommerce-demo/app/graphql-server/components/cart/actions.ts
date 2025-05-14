'use server';

import { graphql } from 'gql.tada';
import { gql } from 'lib/gql';
import { CartModalFragment } from './fragments';

const AddToCartMutation = graphql(
  `
    mutation AddToCartMutation($productId: ID!, $quantity: Int!) {
      addToCart(productId: $productId, quantity: $quantity) {
        ...CartModalFragment
      }
    }
  `,
  [CartModalFragment]
);

export async function addToCart(_prevState: any, productId: string) {
  try {
    const { data } = await gql(AddToCartMutation, { productId, quantity: 1 });
    if (!data?.addToCart) return 'Must be logged in to add products to cart';
  } catch (error) {
    return error instanceof Error ? error.message : `${error}`;
  }
}

const RemoveFromCartMutation = graphql(
  `
    mutation RemoveFromCartMutation($productId: ID!, $quantity: Int) {
      removeFromCart(productId: $productId, quantity: $quantity) {
        ...CartModalFragment
      }
    }
  `,
  [CartModalFragment]
);

export async function removeFromCart(_prevState: any, productId: string) {
  try {
    const { data } = await gql(RemoveFromCartMutation, { productId });
    if (!data?.removeFromCart) return 'Must be logged in to remove products from cart';
  } catch (error) {
    return error instanceof Error ? error.message : `${error}`;
  }
}

export async function updateItemQuantity(
  _prevState: any,
  {
    productId,
    type
  }: {
    productId: string;
    type: 'plus' | 'minus';
  }
) {
  try {
    if (type === 'plus') {
      const { data } = await gql(AddToCartMutation, { productId, quantity: 1 });
      if (!data?.addToCart) return 'Must be logged in to add products to cart';
    } else {
      const { data } = await gql(RemoveFromCartMutation, { productId, quantity: 1 });
      if (!data?.removeFromCart) return 'Must be logged in to remove products from cart';
    }
  } catch (error) {
    return error instanceof Error ? error.message : `${error}`;
  }
}
